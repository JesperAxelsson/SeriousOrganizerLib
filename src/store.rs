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

use time::PreciseTime;
//use schema::*;


pub struct Store {
    pub   entriesCache: Vec<Entry>,
    filesCache: HashMap<i32, Vec<File>>,
    labelsCache: Vec<Label>,
}

impl Store {
    pub fn init() -> Store {
        Store {
            entriesCache: Vec::new(),
            filesCache: HashMap::new(),
            labelsCache: Vec::new(),
        }
    }

    pub fn load_from_store(&mut self) {
        let conn = self.establish_connection();
//        conn.execute("DELETE FROM entries").unwrap();

        self.entriesCache = e::entries.load(&conn).expect("Failed to load entries");
        self.load_files(&conn);
        self.labelsCache = l::labels.load(&conn).expect("Failed to load labels");
    }

    fn load_files(&mut self, connection: &SqliteConnection) {
        let files: Vec<File> = f::files.load(connection).expect("Failed to load files");

        for entry in self.entriesCache.iter() {
            self.filesCache.insert(entry.id, Vec::new());
        }

        for file in files {
            let files = self.filesCache.get_mut(&file.entry_id).expect("Did not find files that really should be there");
            files.push(file);
        }

        println!("Now got files for entries {}", self.filesCache.len());
    }

    pub fn update(&mut self, dir_entries: &Vec<DirEntry>) {
        use std::collections::HashMap;
        use std::collections::HashSet;

        println!("Starting update");
        let start = PreciseTime::now();

        let mut dir_hash = HashMap::with_capacity(dir_entries.len());
        let mut file_hash = HashMap::with_capacity(dir_entries.len());
        for dir in dir_entries.iter() {
            for file in dir_entries.iter() {
                file_hash.insert(&file.path, file);
            }

            dir_hash.insert(&dir.path, dir);
        }

        let connection = self.establish_connection();

        let mut collisions = HashSet::new();
        for entry in self.entriesCache.iter() {
            if let Some(dir_entry) = dir_hash.get(&entry.path) {
                // Update existing entries
                collisions.insert(entry.path.clone());
                let new_size = dir_entry.size as i64;
                if entry.size != new_size {
                    println!("Update entry: {} {}", entry.path, entry.name);
                    diesel::update(entry).set(e::size.eq(new_size)).execute(&connection).expect("Failed to update entry");
                }
            } else {
                // Delete entries not in entries
                println!("Delete entry: {} {}", entry.path, entry.name);
                diesel::delete(e::entries.filter(e::id.eq(entry.id))).execute(&connection).expect("Failed to delete entry");
            }
        }

        // Insert new entries
        let mut insert_query = Vec::with_capacity(dir_hash.len());
        for (key, value) in dir_hash.iter() {
            // Insert
            if !collisions.contains(key.clone()) {
                println!("Insert entry: {}", key);
                insert_query.push((e::name.eq(&value.name), e::path.eq(&value.path), e::size.eq(value.size as i64)));
            }
        }

        diesel::insert_into(e::entries)
            .values(&insert_query)
            .execute(&connection).expect("Failed to execute entry insert query");


        println!("Entries: {} dirs: {}", self.entriesCache.len(), dir_hash.len());
        // Reload entries cache
        self.entriesCache = e::entries.load(&connection).expect("Failed to load entries");

        println!("Entries: {} dirs: {}", self.entriesCache.len(), dir_hash.len());

        // *** Start files updates ***
        let mut insert_query = Vec::new();


        for entry in self.entriesCache.iter() {
            let dir = dir_hash.get(&entry.path).expect(&format!("Failed to find dir when updating files: {}", entry.path));

            let mut file_lookup = HashSet::new();

            if let Some(file_cache) = self.filesCache.get(&entry.id) {
                // Entry already exists

                let mut file_hash = HashMap::new();
                for file in dir.files.iter() {
                    file_hash.insert(file.path.clone(), file);
                }

                for file in file_cache.iter() {
                    file_lookup.insert(file.path.clone());

                    if let Some(oldFile) = file_hash.get(&file.path) {
                        // File exists, check for diffs
                        let new_size = oldFile.size as i64;
                        if file.size != new_size {
                            println!("Update file: {}", oldFile.path);
                            diesel::update(file).set(f::size.eq(new_size)).execute(&connection).expect("Failed to update file");
                        }
                    } else {
                        // File were removed
                        println!("Delete file: {}", entry.path);
                        diesel::delete(f::files.filter(f::id.eq(file.id))).execute(&connection).expect("Failed to delete entry");
                    }
                }
            }

            // Entry is new, insert all files
            for file in dir.files.iter() {
                if !file_lookup.contains(&file.path) {
                    println!("Insert file: {}", file.path);
                    insert_query.push((f::entry_id.eq(entry.id), f::name.eq(&file.name), f::path.eq(&file.path), f::size.eq(file.size as i64)));
                }
            }
        }

        diesel::insert_into(f::files)
            .values(&insert_query)
            .execute(&connection).expect("Failed to execute file insert query");

        self.load_files(&connection);

        // Done!
        println!("Found {:?} dirs, {:?} files and {:?} collisions. Diff: {}", dir_hash.len(), file_hash.len(), collisions.len(), self.entriesCache.len());
        let end = PreciseTime::now();

        println!(
            "Update took: {:?} ms",
            start.to(end).num_milliseconds()
        );
    }

    pub fn get_all_entries(&self) -> &Vec<Entry> {
        return &self.entriesCache;
    }

    pub fn get_files(&self, entry: &Entry) -> Option<&Vec<File>> {
        return self.filesCache.get(&entry.id);
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
        let ret = diesel::insert_into(e::entries)
            .values((e::name.eq("Phillies"), e::path.eq("D:\\temp"), e::size.eq(443)))
            .execute(&connection).expect("Failed to execute query");

        let entries2: Vec<Entry> = e::entries.load(&connection).unwrap();
        println!("Got entries: {}", entries2.len());
        for e in &entries2 {
            println!("{:?}", e);
        }
    }
}
