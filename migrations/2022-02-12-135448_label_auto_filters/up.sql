-- Your SQL goes here
CREATE TABLE label_auto_filters (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    filter TEXT NOT NULL,
    label_id INTEGER NOT NULL,
    FOREIGN KEY(label_id) REFERENCES labels(id) ON DELETE CASCADE
);