-- Your SQL goes here
CREATE TABLE todos (
  id INTEGER PRIMARY KEY NOT NULL,
  content TEXT NOT NULL,
  deadline DATETIME,
  scheduled DATETIME,
  effort INTEGER,
  room TEXT NOT NULL
)
