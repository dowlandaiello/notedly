table! {
    boards (id) {
        id -> Text,
        user_id -> Nullable<Int4>,
        title -> Text,
        visibility -> Int2,
        permissions -> Nullable<Jsonb>,
    }
}

table! {
    notes (id) {
        id -> Text,
        user_id -> Nullable<Int4>,
        title -> Text,
        body -> Text,
    }
}

table! {
    users (email) {
        email -> Text,
        id -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(
    boards,
    notes,
    users,
);
