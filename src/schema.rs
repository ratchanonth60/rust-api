// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        id -> Int4,
        name -> Varchar,
        slug -> Varchar,
    }
}

diesel::table! {
    comments (id) {
        id -> Int4,
        content -> Text,
        user_id -> Int4,
        post_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    password_reset_tokens (email) {
        email -> Varchar,
        token -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        content -> Text,
        user_id -> Int4,
        category_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    todos (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Varchar,
        description -> Nullable<Text>,
        completed -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
        role -> Varchar,
    }
}

diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(comments -> users (user_id));
diesel::joinable!(posts -> categories (category_id));
diesel::joinable!(posts -> users (user_id));
diesel::joinable!(todos -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    comments,
    password_reset_tokens,
    posts,
    todos,
    users,
);
