use lucid_macros_common::PrimaryKeyType;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde_tokenstream::ParseWrapper;
use syn::{Data, DataStruct, DeriveInput, Error, Fields, Ident, parse_quote, spanned::Spanned};

#[derive(Debug)]
pub(crate) struct NameValue {
    name: syn::Path,
    _eq_token: syn::token::Eq,
    value: syn::Path,
}

impl syn::parse::Parse for NameValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

/// Looks for a Diesel Meta-style attribute with a particular identifier.
///
/// As an example, for an attribute like `#[diesel(foo = bar)]`, we can find this
/// attribute by calling `get_nv_attr(&item.attrs, "foo")`.
fn get_diesel_nv_attr(attrs: &[syn::Attribute], name: &str) -> Option<NameValue> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("diesel"))
        .filter_map(|attr| attr.parse_args::<NameValue>().ok())
        .find(|nv| nv.name.is_ident(name))
}

/// Looks up a named field within a struct.
fn get_field_with_name<'a>(data: &'a DataStruct, name: &str) -> Option<&'a syn::Field> {
    if let Fields::Named(ref data_fields) = data.fields {
        data_fields.named.iter().find(|field| {
            if let Some(ident) = &field.ident {
                ident == name
            } else {
                false
            }
        })
    } else {
        None
    }
}

#[proc_macro_derive(Resource, attributes(resource))]
pub fn resource_target(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MacroAttributes {
    #[serde(default)]
    uuid_kind: Option<ParseWrapper<syn::Ident>>,
    #[serde(default)]
    deletable: Option<ParseWrapper<syn::LitBool>>,
}

impl MacroAttributes {
    fn parse_from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let inner_attrs = attrs
            .iter()
            .filter(|&attr| attr.path().is_ident("resource"))
            .map(|attr| {
                let meta_list = attr.meta.require_list()?;
                Ok::<_, syn::Error>(&meta_list.tokens)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let tokens = quote! { #(#inner_attrs,)* };
        serde_tokenstream::from_tokenstream(&tokens)
    }

    fn uuid_ty(&self) -> PrimaryKeyType {
        self.uuid_kind.as_ref().map_or_else(
            || PrimaryKeyType::Standard(parse_quote!(::uuid::Uuid)),
            |v| PrimaryKeyType::new_typed_uuid(v),
        )
    }

    fn deletable(&self) -> bool {
        self.deletable.as_ref().map_or_else(|| true, |w| w.value)
    }
}

fn derive_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let item = syn::parse2::<DeriveInput>(input)?;
    let name = &item.ident;

    let table_nv = get_diesel_nv_attr(&item.attrs, "table_name").ok_or_else(|| {
        Error::new(
            item.span(),
            format!(
                "Resource needs 'table_name' attribute.\n\
                    Try adding #[diesel(table_name = your_table_name)] to {}.",
                name,
            ),
        )
    })?;
    let table_name = table_nv.value;

    let input = MacroAttributes::parse_from_attrs(&item.attrs)?;
    let uuid_ty = input.uuid_ty();
    let deletable = input.deletable();

    // Ensure that a field named "identity" exists within this struct.
    if let Data::Struct(ref data) = item.data {
        let field = get_field_with_name(data, "identity")
            .ok_or_else(|| {
                Error::new(
                    item.span(),
                    format!(
                        "{name}Identity must be embedded within {name} as a field named `identity`.\n\
                        This proc macro will try to add accessor methods to {name}; this can only be\n\
                        accomplished if we know where to access them.",
                        name=name,
                    ),
                )
            })?;

        return Ok(build(name, &table_name, &field.ty, &uuid_ty, deletable));
    }

    Err(Error::new(
        item.span(),
        "Resource can only be derived for structs",
    ))
}

fn build(
    struct_name: &Ident,
    table_name: &syn::Path,
    observed_identity_ty: &syn::Type,
    uuid_ty: &PrimaryKeyType,
    deletable: bool,
) -> TokenStream {
    let identity_struct = build_identity(struct_name, table_name, uuid_ty, deletable);
    let resource_impl = build_impl(struct_name, observed_identity_ty, uuid_ty, deletable);

    quote! {
        #identity_struct
        #resource_impl
    }
}

fn build_identity(
    struct_name: &Ident,
    table_name: &syn::Path,
    uuid_ty: &PrimaryKeyType,
    deletable: bool,
) -> TokenStream {
    let identity_doc = format!(
        "Auto-generated identity for [`{}`] from deriving [`macro@Resource`].",
        struct_name,
    );
    let identity_name = format_ident!("{}Identity", struct_name);

    let external_uuid_ty = uuid_ty.external();
    let db_uuid_ty = uuid_ty.db();
    let convert_external_to_db = uuid_ty.external_to_db_lucid_db_models(quote! { id });

    let mut deleted_at_def = quote! {};
    let mut deleted_at_new = quote! {};
    if deletable {
        deleted_at_def = quote! {
            pub deleted_at: ::std::option::Option<::chrono::DateTime<::chrono::Utc>>,
        };

        deleted_at_new = quote! {
            deleted_at: None,
        };
    }

    quote! {
        #[doc = #identity_doc]
        #[derive(Clone, Debug, PartialEq, Selectable, Queryable, Insertable, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        #[diesel(table_name = #table_name)]
        pub struct #identity_name {
            pub id: #db_uuid_ty,
            pub created_at: ::chrono::DateTime<::chrono::Utc>,
            pub updated_at: ::chrono::DateTime<::chrono::Utc>,
            #deleted_at_def
        }

        impl #identity_name {
            pub fn new(
                id: #external_uuid_ty,
            ) -> Self {
                let now = ::chrono::Utc::now();
                Self {
                    id: #convert_external_to_db,
                    created_at: now,
                    updated_at: now,
                    #deleted_at_new
                }
            }
        }
    }
}

fn build_impl(
    struct_name: &Ident,
    observed_identity_ty: &syn::Type,
    uuid_ty: &PrimaryKeyType,
    deletable: bool,
) -> TokenStream {
    let identity_trait = format_ident!("__{}IdentityMarker", struct_name);
    let identity_name = format_ident!("{}Identity", struct_name);

    let external_uuid_ty = uuid_ty.external();
    let convert_db_to_external = uuid_ty.db_to_external(quote! { self.identity.id });

    let mut deleted_at_def = quote! { None };
    if deletable {
        deleted_at_def = quote! { self.identity.deleted_at };
    }

    quote! {
        // Verify that the field named "identity" is actually the generated
        // type within the struct deriving Resource
        trait #identity_trait {}
        impl #identity_trait for #identity_name {}
        const _: () = {
            fn assert_identity<T: #identity_trait>() {}
            fn assert_all() {
                assert_identity::<#observed_identity_ty>();
            }
        };

        impl ::lucid_types::identity::Resource for #struct_name {
            type IdType = #external_uuid_ty;

            fn id(&self) -> #external_uuid_ty {
                #convert_db_to_external
            }

            fn created_at(&self) -> ::chrono::DateTime<::chrono::Utc> {
                self.identity.created_at
            }

            fn updated_at(&self) -> ::chrono::DateTime<::chrono::Utc> {
                self.identity.updated_at
            }

            fn deleted_at(&self) -> ::std::option::Option<::chrono::DateTime<::chrono::Utc>> {
                #deleted_at_def
            }
        }
    }
}
