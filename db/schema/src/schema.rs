// @generated automatically by Diesel CLI.

diesel::table! {
    inventory_hosts (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        external_id -> Text,
        system_admin -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        display_name -> Nullable<Text>,
        is_owner -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(inventory_hosts, users,);
