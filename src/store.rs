#![allow(proc_macro_derive_resolution_fallback)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use diesel;
use diesel::prelude::*;
//use diesel::sqlite::SqliteConnection;
//
use crate::models::*;

use crate::schema::entries::dsl as e;
use crate::schema::entry2labels::dsl as e2l;
use crate::schema::files::dsl as f;
use crate::schema::labels::dsl as l;

use std::collections::{HashMap, HashSet};

use time::PreciseTime;
//use schema::*;

pub struct Store {
    pub entriesCache: Vec<Entry>,
    filesCache: HashMap<EntryId, Vec<File>>,
    labelsCache: Vec<Label>,
    labelLookupCache: HashMap<LabelId, HashSet<EntryId>>,
}

impl Store {
    pub fn init() -> Store {
        Store {
            entriesCache: Vec::new(),
            filesCache: HashMap::new(),
            labelsCache: Vec::new(),
            labelLookupCache: HashMap::new(),
        }
    }

    pub fn establish_connection(&self) -> SqliteConnection {
        //        let url = ::std::env::var("DATABASE_URL").expect("Failed to find DATABASE_URL");
        let url = String::from("test.sqlite3");
        let connection = SqliteConnection::establish(&url).expect("Failed to establish connection to sqlite");

        connection
            .execute("PRAGMA foreign_keys = ON")
            .expect("Failed to set pragmas");

        return connection;
    }

    pub fn load_from_store(&mut self) {
        let conn = self.establish_connection();
        //        conn.execute("DELETE FROM entries").unwrap();

        self.entriesCache = e::entries.load(&conn).expect("Failed to load entries");
        self.load_files(&conn);
        self.labelsCache = l::labels.load(&conn).expect("Failed to load labels");
        self.load_labels(&conn);
    }

    fn load_files(&mut self, connection: &SqliteConnection) {
        let files: Vec<File> = f::files.load(connection).expect("Failed to load files");

        for entry in self.entriesCache.iter() {
            self.filesCache.insert(entry.id, Vec::new());
        }

        println!("Got {} files in filecache", self.filesCache.len());

        for file in files {
            let files = self.filesCache.get_mut(&file.entry_id).expect(&format!(
                "Did not find files that really should be there: {:?} path: {:?}",
                file.entry_id, file.path
            ));
            files.push(file);
        }

        println!("Now got files for entries {}", self.filesCache.len());
    }

    fn load_labels(&mut self, connection: &SqliteConnection) {
        let entry2label: Vec<Entry2Label> = e2l::entry2labels
            .load(connection)
            .expect("Failed to load entry mapping");

        let mut lbl_map: HashMap<LabelId, HashSet<EntryId>> = HashMap::new();

        for e2l in entry2label.iter() {
            let set = lbl_map.entry(e2l.label_id).or_insert(HashSet::new());
            set.insert(e2l.entry_id);
        }

        self.labelLookupCache = lbl_map;
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
                    //                    println!("Update entry: {} {}", entry.path, entry.name);
                    diesel::update(entry)
                        .set(e::size.eq(new_size))
                        .execute(&connection)
                        .expect("Failed to update entry");
                }
            } else {
                // Delete entries not in entries
                //                println!("Delete entry: {} {}", entry.path, entry.name);
                diesel::delete(e::entries.filter(e::id.eq(entry.id)))
                    .execute(&connection)
                    .expect("Failed to delete entry");
            }
        }

        // Insert new entries
        let mut insert_query = Vec::with_capacity(dir_hash.len());
        for (key, value) in dir_hash.iter() {
            // Insert
            if !collisions.contains(key.clone()) {
                //                println!("Insert entry: {}", key);
                insert_query.push((
                    e::name.eq(&value.name),
                    e::path.eq(&value.path),
                    e::size.eq(value.size as i64),
                ));
            }
        }

        diesel::insert_into(e::entries)
            .values(&insert_query)
            .execute(&connection)
            .expect("Failed to execute entry insert query");

        // Reload entries cache
        self.entriesCache = e::entries.load(&connection).expect("Failed to load entries");

        //        println!("Entries: {} dirs: {}", self.entriesCache.len(), dir_hash.len());

        // *** Start files updates ***
        let mut insert_query = Vec::new();

        for entry in self.entriesCache.iter() {
            let dir = dir_hash
                .get(&entry.path)
                .expect(&format!("Failed to find dir when updating files: {}", entry.path));

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
                            //                            println!("Update file: {}", oldFile.path);
                            diesel::update(file)
                                .set(f::size.eq(new_size))
                                .execute(&connection)
                                .expect("Failed to update file");
                        }
                    } else {
                        // File were removed
                        //                        println!("Delete file: {}", entry.path);
                        diesel::delete(f::files.filter(f::id.eq(file.id)))
                            .execute(&connection)
                            .expect("Failed to delete entry");
                    }
                }
            }

            // Entry is new, insert all files
            for file in dir.files.iter() {
                if !file_lookup.contains(&file.path) {
                    //                    println!("Insert file: {}", file.path);
                    insert_query.push((
                        f::entry_id.eq(entry.id),
                        f::name.eq(&file.name),
                        f::path.eq(&file.path),
                        f::size.eq(file.size as i64),
                    ));
                }
            }
        }

        diesel::insert_into(f::files)
            .values(&insert_query)
            .execute(&connection)
            .expect("Failed to execute file insert query");

        self.load_files(&connection);

        // Done!
        println!(
            "Found {:?} dirs, {:?} files and {:?} collisions. Diff: {}",
            dir_hash.len(),
            file_hash.len(),
            collisions.len(),
            self.entriesCache.len()
        );
        let end = PreciseTime::now();

        println!("Update took: {:?} ms", start.to(end).num_milliseconds());
    }

    pub fn get_all_entries(&self) -> &Vec<Entry> {
        return &self.entriesCache;
    }

    pub fn get_files(&self, entry: &Entry) -> Option<&Vec<File>> {
        return self.filesCache.get(&entry.id);
    }

    // *** Labels ***
    pub fn set_entry_labels(&mut self, entry_ids: Vec<EntryId>, label_ids: Vec<LabelId>) {
        use diesel::result::Error;

        let mut insert_query = Vec::with_capacity(entry_ids.len() * label_ids.len());

        for entry_id in entry_ids.iter() {

            // Add new labels
            for label_id in label_ids.iter() {
                insert_query.push((
                    e2l::entry_id.eq(entry_id),
                    e2l::label_id.eq(label_id),
                ));
            }
        }

        let connection = self.establish_connection();
        connection.transaction::<_, Error, _>(  || {

            // Remove labels not set
            for slice in entry_ids.iter().collect::<Vec<_>>().chunks(500) {
                diesel::delete(e2l::entry2labels.filter(e2l::entry_id.eq_any(slice.to_vec())))
                    .execute(&connection)?;
            }

            println!("Labels deleted");


            println!("Add labels");

            for slice in insert_query.iter().collect::<Vec<_>>().chunks(5000) {
//                for label_id in slice.iter() {
//                    insert_query.push((
//                        e2l::entry_id.eq(entry_id),
//                        e2l::label_id.eq(label_id),
//                    ));


                diesel::insert_into(e2l::entry2labels)
                    .values(slice.to_vec())
                    .execute(&connection)?;
//                }
            }

//                for entry_id in entry_ids {
//
//                    // Add new labels
//                    let mut insert_query = Vec::new();
//
//                    for label_id in label_ids.iter() {
//                        insert_query.push((
//                            e2l::entry_id.eq(entry_id),
//                            e2l::label_id.eq(label_id),
//                        ));
//                    }
//                }

            Ok(())
        }).expect("Failed to set_entry_labels");

        println!("All labels done");
        self.load_labels(&connection);
    }

    pub fn dir_labels(&self, entry_id: EntryId) -> Vec<LabelId> {
        let mut labels = Vec::new();

        for (label_id, entries) in self.labelLookupCache.iter() {
            if entries.contains(&entry_id) {
                labels.push(*label_id);
            }
        }

        return labels;
    }

    pub fn has_label(&mut self, entry_id: EntryId, label_id: LabelId) -> bool {
        if let Some(entries) = self.labelLookupCache.get(&label_id) {
            return entries.contains(&entry_id);
        }
        return false;
    }

    pub fn add_label(&mut self, name: &str) -> bool {
        if self.labelsCache.iter().any(|lbl| lbl.name == name) {
            return false;
        }

        let connection = self.establish_connection();
        diesel::insert_into(l::labels)
            .values(l::name.eq(name))
            .execute(&connection)
            .expect("Failed to insert new label");
        self.labelsCache = l::labels.load(&connection).expect("Failed to load labels");

        return true;
    }

    pub fn remove_label(&mut self, id: LabelId) {
        unimplemented!("remove_label() is not done yet!")
    }

    pub fn get_all_labels(&self) -> &Vec<Label> {
        return &self.labelsCache;
    }

    // *** Test? ***
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
            .execute(&connection)
            .expect("Failed to execute query");

        let entries2: Vec<Entry> = e::entries.load(&connection).unwrap();
        println!("Got entries: {}", entries2.len());
        for e in &entries2 {
            println!("{:?}", e);
        }
    }
}
