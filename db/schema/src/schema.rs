// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "identity_principal_type"))]
    pub struct IdentityPrincipalType;
}

diesel::table! {
    organisation_roles (id) {
        id -> Uuid,
        organisation_id -> Uuid,
        name -> Text,
        display_name -> Text,
        description -> Nullable<Text>,
        permissions -> Array<Nullable<Text>>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    organisation_users (user_id, organisation_id) {
        user_id -> Uuid,
        organisation_id -> Uuid,
    }
}

diesel::table! {
    organisations (id) {
        id -> Uuid,
        name -> Text,
        display_name -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::IdentityPrincipalType;

    role_bindings (id) {
        id -> Uuid,
        role_name -> Text,
        organisation_id -> Uuid,
        principal_id -> Uuid,
        principal_type -> IdentityPrincipalType,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        external_id -> Nullable<Text>,
        password_hash -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    organisation_roles,
    organisation_users,
    organisations,
    role_bindings,
    users,
);
