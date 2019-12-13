table! {
    boards (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        title -> Text,
        visibility -> Int2,
        permissions -> Nullable<Jsonb>,
    }
}

table! {
    notes (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        title -> Text,
        body -> Text,
    }
}

table! {
    users (oauth_id) {
        oauth_id -> Int4,
        oauth_token -> Nullable<Text>,
        email -> Nullable<Text>,
        id -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(
    boards,
    notes,
    users,
);
