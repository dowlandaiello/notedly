table! {
    boards (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Text,
        visibility -> Int2,
    }
}

table! {
    notes (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        title -> Text,
        body -> Text,
    }
}

table! {
    permissions (user_id) {
        user_id -> Int4,
        board_id -> Int4,
        read -> Bool,
        write -> Bool,
    }
}

table! {
    users (oauth_id) {
        oauth_id -> Int4,
        oauth_token -> Text,
        email -> Text,
        id -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(
    boards,
    notes,
    permissions,
    users,
);
