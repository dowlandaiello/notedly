CREATE TABLE users (
    -- The user's oauth identifier
    oauth_id INTEGER PRIMARY KEY, 

    -- The user's current oauth access token hash
    oauth_token TEXT,

    -- The user's email
    email TEXT,

    -- The user's ID
    id SERIAL
);

CREATE TABLE boards (
    -- The hash of the board's name and owner
    id SERIAL PRIMARY KEY,

    -- The ID of the board's owner
    user_id INTEGER,

    -- The title of the board
    title TEXT NOT NULL,

    -- The privacy setting of the board (0 => private, 1 => unlisted, 2 => permissive)
    visibility SMALLINT NOT NULL,

    -- The permissions of the board, per each user
    permissions JSONB
);

CREATE TABLE notes (
    -- The hash of the post's name and author
    id SERIAL PRIMARY KEY,

    -- The ID of the board's owner
    user_id INTEGER,

    -- The title of the post
    title TEXT NOT NULL,

    -- The text contained in the post
    body TEXT NOT NULL
);
