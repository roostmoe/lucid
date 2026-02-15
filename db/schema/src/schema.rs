// @generated automatically by Diesel CLI.

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
