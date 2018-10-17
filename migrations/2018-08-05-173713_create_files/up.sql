-- Your SQL goes here

CREATE TABLE files (
  id INTEGER PRIMARY KEY NOT NULL,
  entry_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  size BigInt NOT NULL,
  FOREIGN KEY(entry_id) REFERENCES entries(id) ON DELETE CASCADE
);