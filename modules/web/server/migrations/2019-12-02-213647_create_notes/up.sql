CREATE TABLE users (
    -- The user's ID
    id SERIAL PRIMARY KEY,

    -- The user's oauth identifier
    oauth_id INTEGER NOT NULL UNIQUE,

    -- The user's current oauth access token hash
    oauth_token TEXT NOT NULL,

    -- The user's email
    email TEXT NOT NULL
);

CREATE TABLE boards (
    -- The hash of the board's name and owner
    id SERIAL PRIMARY KEY,

    -- The ID of the board's owner
    user_id INTEGER NOT NULL,

    -- The title of the board
    title TEXT NOT NULL,

    -- The privacy setting of the board (0 => private, 1 => unlisted, 2 => permissive)
    visibility SMALLINT NOT NULL
);

CREATE TABLE notes (
    -- The hash of the post's name and author
    id SERIAL PRIMARY KEY,

    -- The ID of the post's owner
    user_id INTEGER NOT NULL,

    -- The ID of the post's parent board
    board_id INTEGER NOT NULL,

    -- The title of the post
    title TEXT NOT NULL,

    -- The text contained in the post
    body TEXT NOT NULL
);

CREATE TABLE permissions (
    -- The ID of the permission
    id SERIAL PRIMARY KEY,

    -- The ID of the associated user targeted by the permission
    user_id INTEGER NOT NULL,

    -- The ID of the parent board / post (n)
    board_id INTEGER NOT NULL,   

    -- Can the user read the posts on the board?
    read BOOLEAN NOT NULL,

    -- Can the user write new posts to the board?
    write BOOLEAN NOT NULL
);
