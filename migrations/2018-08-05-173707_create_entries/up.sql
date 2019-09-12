-- Your SQL goes here

CREATE TABLE entries (
  id INTEGER PRIMARY KEY NOT NULL,
  location_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  size BigInt NOT NULL,
  FOREIGN KEY(location_id) REFERENCES locations(id) ON DELETE CASCADE
);
