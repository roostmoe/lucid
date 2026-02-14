// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        external_id -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        display_name -> Nullable<Text>,
        is_owner -> Bool,
    }
}
