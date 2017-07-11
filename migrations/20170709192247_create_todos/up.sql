-- Your SQL goes here
CREATE TABLE todos (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    deadline TEXT,
    scheduled TEXT,
    effort INTEGER,
    room TEXT NOT NULL
)