-- Your SQL goes here
CREATE TABLE entry2labels (
    entry_id INTEGER NOT NULL,
    label_id INTEGER NOT NULL,
    PRIMARY KEY (entry_id, label_id),
    FOREIGN KEY(entry_id) REFERENCES entries(id) ON DELETE CASCADE,
    FOREIGN KEY(label_id) REFERENCES labels(id) ON DELETE CASCADE
);