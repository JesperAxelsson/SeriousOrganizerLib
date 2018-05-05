//use time;
//use time::Timespec;
use rusqlite::Connection;


#[derive(Debug)]
pub struct Label {
    id: i32,
    pub name: String,
}

#[derive(Debug)]
pub struct Entry {
    id: i32,
    pub path: String,
}

fn get_connection() -> Connection {
    let conn = Connection::open("mydb.db").expect("Failed to open database");
    conn.execute("PRAGMA foreign_keys = ON", &[]).unwrap();

    conn
}

pub fn create_tables() {
    let conn = get_connection();

    conn.execute("CREATE TABLE IF NOT EXISTS label (
                  label_id        INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL
                  )", &[]).unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS entry (
                  entry_id        INTEGER PRIMARY KEY,
                  path            TEXT NOT NULL
                  )", &[]).unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS entry2label (
                  entry_id INTEGER,
                  label_id INTEGER,
                  FOREIGN KEY(entry_id) REFERENCES entry(entry_id) ON DELETE CASCADE,
                  FOREIGN KEY(label_id) REFERENCES label(label_id) ON DELETE CASCADE
                  )", &[]).unwrap();
}

pub fn add_label(label: &str) {
    let conn = get_connection();
    conn.execute("INSERT INTO label (name)
                 VALUES (?1)",
                 &[&label]).unwrap();
}

pub fn get_labels() -> Vec<Label> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT label_id, name FROM label").unwrap();

    let label_iter = stmt.query_map(&[], |row| {
        Label {
            id: row.get(0),
            name: row.get(1),
        }
    }).unwrap();

    let mut labels = Vec::new();
    for lbl in label_iter {
        labels.push(lbl.unwrap());
    }

    labels
}


pub fn add_entry(path: &str) {
    let conn = get_connection();
    conn.execute("INSERT INTO entry (path)
                 VALUES (?1)",
                 &[&path]).unwrap();
}


pub fn get_entries() -> Vec<Entry> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT entry_id, path FROM entry").unwrap();

    let entry_iter = stmt.query_map(&[], |row| {
        Entry {
            id: row.get(0),
            path: row.get(1),
        }
    }).unwrap();

    let mut entries = Vec::new();
    for entry in entry_iter {
        entries.push(entry.unwrap());
    }

    entries
}


pub fn add_entry_label(path: & str, label: &Label) {
    let entries = get_entries();
    let entry_id = match entries.iter().find(|e| e.path == path) {
        Some(entry) => entry.id,
        None => {
            add_entry(&path);
            add_entry_label(&path, label);
            println!("Added label: {:?}", path);
            return;
        }
    };

    let conn = get_connection();
    conn.execute("INSERT INTO entry2label (entry_id, label_id)
                 VALUES (?1, ?2)",
                 &[&entry_id, &(*label).id]).unwrap();

}