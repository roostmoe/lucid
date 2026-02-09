use lucid_macros_common::PrimaryKeyType;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde_tokenstream::ParseWrapper;
use syn::parse_quote;

extern crate proc_macro;

#[proc_macro]
pub fn authz_resource(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match do_authz_resource(input.into()) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[derive(serde::Deserialize, Debug)]
struct Input {
    /// Name of the resource
    name: String,

    /// Name of the parent `authz` resource
    parent: String,

    /// Rust type for the primary key for this resource.
    primary_key: InputPrimaryKeyType,

    /// The `TypedUuidKind` for this resource. Must be exclusive.
    #[serde(default)]
    input_key: Option<ParseWrapper<syn::Type>>,

    /// Whether roles may be attached directly to this resource
    roles_allowed: bool,

    /// How to generate the Polar snippet for this resource
    polar_snippet: PolarSnippet,
}

#[derive(Debug)]
struct InputPrimaryKeyType(PrimaryKeyType);

impl<'de> serde::Deserialize<'de> for InputPrimaryKeyType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Attempt to parse as either a string or a map.
        struct PrimaryKeyVisitor;

        impl<'de2> serde::de::Visitor<'de2> for PrimaryKeyVisitor {
            type Value = PrimaryKeyType;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                formatter.write_str(
                    "a Rust type, or a map with a single key `uuid_kind`",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                syn::parse_str(value)
                    .map(PrimaryKeyType::Standard)
                    .map_err(|e| E::custom(e.to_string()))
            }

            // seq represents a tuple type
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de2>,
            {
                let mut elements = vec![];
                while let Some(element) =
                    seq.next_element::<ParseWrapper<syn::Type>>()?
                {
                    elements.push(element.into_inner());
                }
                Ok(PrimaryKeyType::Standard(parse_quote!((#(#elements,)*))))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de2>,
            {
                let key: String = map.next_key()?.ok_or_else(|| {
                    serde::de::Error::custom("expected a single key")
                })?;
                if key == "uuid_kind" {
                    // uuid kinds must be plain identifiers
                    let value: ParseWrapper<syn::Ident> = map.next_value()?;
                    Ok(PrimaryKeyType::new_typed_uuid(&value))
                } else {
                    Err(serde::de::Error::custom(
                        "expected a single key `uuid_kind`",
                    ))
                }
            }
        }

        deserializer.deserialize_any(PrimaryKeyVisitor).map(InputPrimaryKeyType)
    }
}

#[derive(serde::Deserialize, Debug)]
enum PolarSnippet {
    /// Don't generate it at all -- it's generated elsewhere
    Custom,

    /// Generate it as a resource nested under a tenant
    InTenant,
}

fn do_authz_resource(
    raw_input: TokenStream
) -> Result<TokenStream, syn::Error> {
    let input = serde_tokenstream::from_tokenstream::<Input>(&raw_input)?;
    let resource_name = format_ident!("{}", input.name);
    let parent_resource_name = format_ident!("{}", input.name);
    let parent_as_snake = heck::AsSnakeCase(&input.parent).to_string();
    let primary_key_type = input.primary_key.0.external();
    let input_key_type = input.input_key.as_deref().unwrap_or(primary_key_type);

    let (has_role_body, as_roles_body, api_resource_roles_trait) =
        if input.roles_allowed {
            (
                quote! {
                    actor.has_role_resource(
                        ResourceType::#resource_name,
                        r.key,
                        &role
                    )
                },
                quote! { Some(self) },
                quote! {
                    impl ApiResourceWithRoles for #resource_name {
                        fn resource_id(&self) -> Uuid {
                            self.key
                        }

                        fn conferred_roles_by(
                            &self,
                            _authn: &authn::Context,
                        ) ->
                            Result<
                                Option<(
                                    ResourceType,
                                    Uuid,
                                )>,
                                Error,
                            >
                        {
                            Ok(None)
                        }
                    }
                },
            )
        } else {
            (quote! { false }, quote! { None }, quote! {})
        };

    let polar_snippet = match (input.polar_snippet, input.parent.as_str()) {
        (PolarSnippet::Custom, _) => String::new(),

        (PolarSnippet::InTenant, _) => format!(
            r#"
                resource {} {{
                    permissions = [
                        "list",
                        "get",
                        "create",
                        "update",
                        "delete"
                    ];

                    relations = {{ containing_org: Organisation }};
                    "list" if "viewer" on "containing_org";
                    "get" if "viewer" on "containing_org";
                    "create" if "admin" on "containing_org";
                    "update" if "admin" on "containing_org";
                    "delete" if "admin" on "containing_org";
                }}

                has_relation(parent: Organisation, "containing_org", child: {})
                    if child.organisation = parent;
            "#,
            resource_name, resource_name,
        ),
    };

    let doc_struct = format!(
        "`authz` type for a resource of type {}\
        \
        Used to uniquely identify a resource of type {} across renames, moves,\
        etc., and to do authorization checks (see \
        [`crate::context::OpContext::authorize()`]). See [`crate::authz`] \
        module-level documentation for more information.",
        resource_name, resource_name,
    );

    Ok(quote! {
        #[doc = #doc_struct]
        #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
        pub struct #resource_name {
            parent: #parent_resource_name,
            key: #primary_key_type,
            lookup_type: LookupType,
        }

        impl #resource_name {
            /// Makes a new `authz` struct for this resource with the given
            /// `parent`, unique key `key`, looked up as described by
            /// `lookup_type`
            pub fn new(
                parent: #parent_resource_name,
                key: #input_key_type,
                lookup_type: LookupType,
            ) -> #resource_name {
                #resource_name {
                    parent,
                    key.into(),
                    lookup_type,
                }
            }

            pub fn with_primary_key(
                parent: #parent_resource_name,
                key: #primary_key_type,
                lookup_type: LookupType,
            ) -> #resource_name {
                #resource_name {
                    parent,
                    key,
                    lookup_type,
                }
            }

            pub fn id(&self) -> #primary_key_type {
                self.key.clone().into()
            }

            pub(super) fn init() -> Init {
                use oso::PolarClass;
                Init {
                    polar_snippet: #polar_snippet,
                    polar_class: #resource_name::get_polar_class(),
                }
            }
        }

        impl Eq for #resource_name {}
        impl PartialEq for #resource_name {
            fn eq(&self, other: &Self) -> bool {
                self.key == other.key
            }
        }

        impl oso::PolarClass for #resource_name {
            fn get_polar_class_builder() -> oso::ClassBuilder<Self> {
                oso::Class::builder()
                    .with_equality_check()
                    .add_method(
                        "has_role",
                        |
                            r: &#resource_name,
                            actor: AuthenticatedActor,
                            role: String
                        | { #has_role_body },
                    )
                    .add_attribute_getter(
                        #parent_as_snake,
                        |r: &#resource_name| r.parent.clone()
                    )
            }
        }
    })
}
