// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "identity_principal_type"))]
    pub struct IdentityPrincipalType;
}

diesel::table! {
    console_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        organisation_id -> Uuid,
        token -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_seen_at -> Timestamptz,
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
        resource_id -> Uuid,
        resource_type -> Text,
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
        system_admin -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(console_sessions -> organisations (organisation_id));
diesel::joinable!(console_sessions -> users (user_id));
diesel::joinable!(organisation_users -> organisations (organisation_id));
diesel::joinable!(organisation_users -> users (user_id));
diesel::joinable!(role_bindings -> organisations (organisation_id));

diesel::allow_tables_to_appear_in_same_query!(
    console_sessions,
    organisation_users,
    organisations,
    role_bindings,
    users,
);
