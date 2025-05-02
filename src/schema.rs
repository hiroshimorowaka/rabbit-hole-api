// @generated automatically by Diesel CLI.

diesel::table! {
    files (id) {
        id -> Integer,
        filename -> Text,
        filepath -> Text,
        uploader_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        password -> Text,
        group_name -> Text,
    }
}

diesel::joinable!(files -> users (uploader_id));

diesel::allow_tables_to_appear_in_same_query!(files, users,);
