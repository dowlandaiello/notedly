CREATE TABLE users (
    -- The user's email
    email TEXT PRIMARY KEY, 

    -- A hash of the user's oauth access token
    id TEXT NOT NULL
);

CREATE TABLE boards (
    -- The hash of the board's name and owner
    id CHAR(64) PRIMARY KEY,

    -- The email address of the owner of the board
    owner TEXT NOT NULL, 

    -- The name of the board
    title TEXT NOT NULL,

    -- The privacy setting of the board (0 => private, 1 => unlisted, 2 => permissive)
    visibility SMALLINT NOT NULL,

    -- The permissions of the board, per each user
    permissions JSONB
);

CREATE TABLE notes (
    -- The hash of the post's name and author
    id CHAR(64) PRIMARY KEY,

    -- The author of the post
    author TEXT NOT NULL,

    -- The title of the post
    title TEXT NOT NULL,

    -- The text contained in the post
    body TEXT NOT NULL
);
