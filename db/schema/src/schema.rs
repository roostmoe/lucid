use diesel::table;

pub mod sql_types {
    use diesel::sql_types::SqlType;

    #[derive(SqlType)]
    #[diesel(postgres_type(name = "identity_principal_type"))]
    pub struct IdentityPrincipalType;
}

table! {
    organisations (id) {
        id -> Uuid,
        name -> Text,
        display_name -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        external_id -> Nullable<Text>,
        password_hash -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    organisation_users (user_id, organisation_id) {
        user_id -> Uuid,
        organisation_id -> Uuid,
    }
}

table! {
    organisation_roles (id) {
        id -> Uuid,
        organisation_id -> Uuid,
        name -> Text,
        display_name -> Text,
        description -> Nullable<Text>,
        permissions -> Array<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

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
