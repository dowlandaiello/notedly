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
    permissions (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        read -> Bool,
        write -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        oauth_id -> Int4,
        oauth_token -> Text,
        email -> Text,
    }
}

allow_tables_to_appear_in_same_query!(boards, notes, permissions, users,);
