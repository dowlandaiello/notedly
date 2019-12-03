table! {
    boards (id) {
        id -> Bpchar,
        owner -> Text,
        title -> Text,
        visibility -> Int2,
        permissions -> Nullable<Jsonb>,
    }
}

table! {
    notes (id) {
        id -> Bpchar,
        author -> Text,
        title -> Text,
        body -> Text,
    }
}

table! {
    users (email) {
        email -> Text,
        id -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    boards,
    notes,
    users,
);
