-- Your SQL goes here
CREATE TABLE location2entries (
    location_id INTEGER NOT NULL,
    entry_id INTEGER NOT NULL,
    PRIMARY KEY (location_id, entry_id),
    FOREIGN KEY(entry_id) REFERENCES entries(id) ON DELETE CASCADE,
    FOREIGN KEY(location_id) REFERENCES locations(id) ON DELETE CASCADE
);