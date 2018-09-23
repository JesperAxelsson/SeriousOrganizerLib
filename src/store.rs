#![allow(proc_macro_derive_resolution_fallback)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use diesel;
use diesel::prelude::*;
//use diesel::sqlite::SqliteConnection;
//
use models::*;
use schema::entries::dsl as e;
use schema::files::dsl as f;
use schema::labels::dsl as l;
use std::collections::HashMap;

//use schema::*;


pub struct Store {
    entriesCache: Vec<Entry>,
    filesCache: Vec<File>,
    labelsCache: Vec<Label>,
}

impl Store {
    pub fn init() -> Store {
        Store {
            entriesCache: Vec::new(),
            filesCache: Vec::new(),
            labelsCache: Vec::new(),
        }
    }

    pub fn load_from_store(&mut self) {
        let conn = self.establish_connection();
//        conn.execute("DELETE FROM entries").unwrap();

        self.entriesCache = e::entries.load(&conn).expect("Failed to load entries");
        self.filesCache = f::files.load(&conn).expect("Failed to load files");
        self.labelsCache = l::labels.load(&conn).expect("Failed to load labels");
    }

    pub fn update(&mut self, dir_entries: Vec<DirEntry>) {
        use std::collections::HashMap;
        use std::collections::HashSet;

        let mut dir_hash = HashMap::with_capacity(dir_entries.len());
        let mut file_hash = HashMap::with_capacity(dir_entries.len());
        for dir in dir_entries.iter() {
            for file in dir_entries.iter() {
                file_hash.insert(&file.path, file);
            }

            dir_hash.insert(&dir.path, dir);
        }

        let mut store_hash = HashSet::with_capacity(self.entriesCache.len());
        for dir in self.entriesCache.iter() {
            store_hash.insert(&dir.path);
        }

        for entry in self.entriesCache.iter() {
            if !dir_hash.contains_key(&entry.path) {
                // Update

                dir_hash.remove(&entry.path);
            }
        }

        for (key, value) in dir_hash.iter() {
            //
            
        }

        println!("Found {:?} dirs, {:?} files and {:?} entries. Diff: {}", dir_hash.len(), file_hash.len(), store_hash.len(), 0);
    }

    pub fn get_url(&self) {
        let url = String::from("test.sqlite3");

        println!("Url: {:?}", url);
        //        let url = ::std::env::var("DATABASE_URL").expect("Failed to find DATABASE_URL");
        //        SqliteConnection::establish(&url).expect("Failed to establish connection to sqlite")
    }

    pub fn establish_connection(&self) -> SqliteConnection {
        //        let url = ::std::env::var("DATABASE_URL").expect("Failed to find DATABASE_URL");
        let url = String::from("test.sqlite3");
        SqliteConnection::establish(&url).expect("Failed to establish connection to sqlite")
    }

    pub fn test_db(&mut self) {
        //        use schema::entries::dsl::*;


        println!("Get connection");
        let connection = self.establish_connection();
        //        pub name: String,
        //        pub path: String,
        //        pub size: u64,
        //        println!("Insert stuff");
        diesel::insert_into(e::entries)
            .values((e::name.eq("Phillies"), e::path.eq("D:\\temp"), e::size.eq(443)))
            .execute(&connection).expect("Failed to execute query");

        let entries2: Vec<Entry> = e::entries.load(&connection).unwrap();
        println!("Got entries: {}", entries2.len());
        for e in &entries2 {
            println!("{:?}", e);
        }
    }
}

////use time;
////use time::Timespec;
//use rusqlite::Connection;
//
//
//#[derive(Debug)]
//pub struct Label {
//    id: i32,
//    pub name: String,
//}
//
//#[derive(Debug)]
//pub struct Entry {
//    id: i32,
//    pub path: String,
//}
//
//fn get_connection() -> Connection {
//    let conn = Connection::open("mydb.db").expect("Failed to open database");
//    conn.execute("PRAGMA foreign_keys = ON", &[]).unwrap();
//
//    conn
//}
//
//pub fn create_tables() {
//    let conn = get_connection();
//
//    conn.execute("CREATE TABLE IF NOT EXISTS label (
//                  label_id        INTEGER PRIMARY KEY,
//                  name            TEXT NOT NULL
//                  )", &[]).unwrap();
//
//    conn.execute("CREATE TABLE IF NOT EXISTS entry (
//                  entry_id        INTEGER PRIMARY KEY,
//                  path            TEXT NOT NULL
//                  )", &[]).unwrap();
//
//    conn.execute("CREATE TABLE IF NOT EXISTS entry2label (
//                  entry_id INTEGER,
//                  label_id INTEGER,
//                  FOREIGN KEY(entry_id) REFERENCES entry(entry_id) ON DELETE CASCADE,
//                  FOREIGN KEY(label_id) REFERENCES label(label_id) ON DELETE CASCADE
//                  )", &[]).unwrap();
//}
//
//pub fn add_label(label: &str) {
//    let conn = get_connection();
//    conn.execute("INSERT INTO label (name)
//                 VALUES (?1)",
//                 &[&label]).unwrap();
//}
//
//pub fn get_labels() -> Vec<Label> {
//    let conn = get_connection();
//    let mut stmt = conn.prepare("SELECT label_id, name FROM label").unwrap();
//
//    let label_iter = stmt.query_map(&[], |row| {
//        Label {
//            id: row.get(0),
//            name: row.get(1),
//        }
//    }).unwrap();
//
//    let mut labels = Vec::new();
//    for lbl in label_iter {
//        labels.push(lbl.unwrap());
//    }
//
//    labels
//}
//
//
//pub fn add_entry(path: &str) {
//    let conn = get_connection();
//    conn.execute("INSERT INTO entry (path)
//                 VALUES (?1)",
//                 &[&path]).unwrap();
//}
//
//
//pub fn get_entries() -> Vec<Entry> {
//    let conn = get_connection();
//    let mut stmt = conn.prepare("SELECT entry_id, path FROM entry").unwrap();
//
//    let entry_iter = stmt.query_map(&[], |row| {
//        Entry {
//            id: row.get(0),
//            path: row.get(1),
//        }
//    }).unwrap();
//
//    let mut entries = Vec::new();
//    for entry in entry_iter {
//        entries.push(entry.unwrap());
//    }
//
//    entries
//}
//
//
//pub fn add_entry_label(path: & str, label: &Label) {
//    let entries = get_entries();
//    let entry_id = match entries.iter().find(|e| e.path == path) {
//        Some(entry) => entry.id,
//        None => {
//            add_entry(&path);
//            add_entry_label(&path, label);
//            println!("Added label: {:?}", path);
//            return;
//        }
//    };
//
//    let conn = get_connection();
//    conn.execute("INSERT INTO entry2label (entry_id, label_id)
//                 VALUES (?1, ?2)",
//                 &[&entry_id, &(*label).id]).unwrap();
//
//}
