use diesel::{allow_tables_to_appear_in_same_query, joinable, table};

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "identity_type"))]
    pub struct DbIdentityType;
}

table! {
    use diesel::sql_types::*;
    use super::sql_types::DbIdentityType;

    sessions (id) {
        id -> Uuid,
        identity_type -> DbIdentityType,
        identity_id -> Uuid,
        refresh_hash -> Text,
        family_id -> Text,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,

        email -> Text,
        display_name -> Text,
    }
}

table! {
    user_password_hashes (user_id) {
        user_id -> Uuid,
        hash -> Text,
    }
}

table! {
    groups (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,

        display_name -> Text,
    }
}

table! {
    group_memberships (group_id, user_id) {
        group_id -> Uuid,
        user_id -> Uuid,
    }
}

joinable!(user_password_hashes -> users (user_id));
joinable!(group_memberships -> groups (group_id));
joinable!(group_memberships -> users (user_id));

allow_tables_to_appear_in_same_query!(
    users,
    groups,
    group_memberships,
);
allow_tables_to_appear_in_same_query!(users, user_password_hashes);
