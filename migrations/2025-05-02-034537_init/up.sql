CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    group_name TEXT NOT NULL
);

CREATE TABLE files (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL,
    uploader_id INTEGER NOT NULL,
    FOREIGN KEY(uploader_id) REFERENCES users(id)
);

