#![allow(proc_macro_derive_resolution_fallback)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use diesel::connection::SimpleConnection;
use log::{debug, error, info, trace, warn};

use diesel;
use diesel::prelude::*;

use diesel_migrations::{self, EmbeddedMigrations, MigrationHarness};

use crate::models::*;

use crate::schema::entries::dsl as e;
use crate::schema::entry2labels::dsl as e2l;
use crate::schema::files::dsl as f;
use crate::schema::label_auto_filters::dsl as aut;
use crate::schema::labels::dsl as l;
use crate::schema::locations::dsl as loc;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use time::Instant;
//use schema::*;

// TODO: Enable migrations again
// embed_migrations!();
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct Store {
    db_url: String,
    pub entriesCache: Vec<Entry>,
    filesCache: HashMap<i32, Vec<File>>,
    labelsCache: Vec<Label>,
    labelLookupCache: HashMap<i32, HashSet<i32>>,
    entryLabelLookup: HashMap<i32, HashSet<i32>>,
}

impl Store {
    pub fn init(db_url: &str) -> Store {
        use std::fs::File;
        use std::path::Path;

        let db_path = Path::new(db_url);
        if !db_path.exists() {
            File::create(db_path).expect(&format!("Failed to create db_file: {:?}", db_path));
        }

        let store = Store {
            db_url: db_url.to_string(),
            entriesCache: Vec::new(),
            filesCache: HashMap::new(),
            labelsCache: Vec::new(),
            labelLookupCache: HashMap::new(),
            entryLabelLookup: HashMap::new(),
        };

        let mut connection = store.establish_connection();

        match connection.run_pending_migrations(MIGRATIONS) {
            Ok(migs) => {
                if !migs.is_empty() {
                    println!("Got migrations! {:?}", migs)
                } else {
                    println!("No migrations.")
                }
            }
            Err(err) => eprintln!("Failed to run migrations: {}", err),
        }

        store
    }

    pub fn establish_connection(&self) -> SqliteConnection {
        let mut connection = SqliteConnection::establish(&self.db_url)
            .expect("Failed to establish connection to sqlite");

        connection
            .batch_execute("PRAGMA foreign_keys = ON")
            .expect("Failed to set pragmas");

        return connection;
    }

    /*** Load cache ***/
    pub fn load_from_store(&mut self) {
        let mut conn = self.establish_connection();
        //        conn.execute("DELETE FROM entries").unwrap();

        self.entriesCache = e::entries.load(&mut conn).expect("Failed to load entries");
        self.load_files(&mut conn);
        self.labelsCache = l::labels.load(&mut conn).expect("Failed to load labels");
        self.load_labels(&mut conn);

        // Sort entries
        self.entriesCache.sort_by_key(|e| e.id);
    }

    fn load_files(&mut self, connection: &mut SqliteConnection) {
        let files: Vec<File> = f::files.load(connection).expect("Failed to load files");

        for entry in self.entriesCache.iter() {
            self.filesCache.insert(entry.id, Vec::new());
        }

        debug!("Got {} files in filecache", self.filesCache.len());

        for file in files {
            let files = self.filesCache.get_mut(&file.entry_id).expect(&format!(
                "Did not find files that really should be there: {:?} path: {:?}",
                file.entry_id, file.path
            ));
            files.push(file);
        }

        debug!("Now got files for entries {}", self.filesCache.len());
    }

    fn load_labels(&mut self, connection: &mut SqliteConnection) {
        let entry2label: Vec<Entry2Label> = e2l::entry2labels
            .load(connection)
            .expect("Failed to load entry mapping");

        let mut lbl_map: HashMap<i32, HashSet<i32>> = HashMap::new();

        for e2l in entry2label.iter() {
            let set = lbl_map.entry(e2l.label_id).or_insert(HashSet::new());
            set.insert(e2l.entry_id);
        }

        let mut entry_map: HashMap<i32, HashSet<i32>> = HashMap::new();

        for e2l in entry2label.iter() {
            let set = entry_map.entry(e2l.entry_id).or_insert(HashSet::new());
            set.insert(e2l.label_id);
        }

        self.entryLabelLookup = entry_map;
        self.labelLookupCache = lbl_map;
    }

    pub fn update(&mut self, dir_entries: &Vec<(i32, DirEntry)>) {
        use std::collections::HashMap;
        use std::collections::HashSet;

        debug!("Starting update");
        let start = Instant::now();

        let mut dir_hash = HashMap::with_capacity(dir_entries.len());

        for (location_id, dir) in dir_entries.iter() {
            dir_hash.insert(&dir.path, dir);
        }

        let mut connection = self.establish_connection();

        let mut collisions = HashSet::new();
        for entry in self.entriesCache.iter() {
            if let Some(dir_entry) = dir_hash.get(&entry.path) {
                // Update existing entries
                collisions.insert(entry.path.clone());
                let new_size = dir_entry.size as i64;
                if entry.size != new_size {
                    // trace!("Update entry: {} {}", entry.path, entry.name);
                    diesel::update(entry)
                        .set(e::size.eq(new_size))
                        .execute(&mut connection)
                        .expect("Failed to update entry");
                }
            } else {
                // Delete entries not in entries
                //                trace!("Delete entry: {} {}", entry.path, entry.name);
                diesel::delete(e::entries.filter(e::id.eq(entry.id)))
                    .execute(&mut connection)
                    .expect("Failed to delete entry");
            }
        }

        // Insert new entries
        let mut insert_query = Vec::with_capacity(dir_hash.len());
        for (key, value) in dir_hash.iter() {
            // Insert
            if !collisions.contains(key.clone()) {
                //                tracec!("Insert entry: {}", key);
                insert_query.push((
                    e::location_id.eq(value.location_id),
                    e::name.eq(&value.name),
                    e::path.eq(&value.path),
                    e::size.eq(value.size as i64),
                ));
            }
        }

        diesel::insert_into(e::entries)
            .values(&insert_query)
            .execute(&mut connection)
            .expect("Failed to execute entry insert query");

        // Reload entries cache
        self.entriesCache = e::entries
            .load(&mut connection)
            .expect("Failed to load entries");

        //        debug!("Entries: {} dirs: {}", self.entriesCache.len(), dir_hash.len());

        // *** Start files updates ***
        let mut insert_query = Vec::new();

        for entry in self.entriesCache.iter() {
            let dir = dir_hash.get(&entry.path).expect(&format!(
                "Failed to find dir when updating files: {}",
                entry.path
            ));

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
                            trace!("Update file: {}", oldFile.path);
                            diesel::update(file)
                                .set(f::size.eq(new_size))
                                .execute(&mut connection)
                                .expect("Failed to update file");
                        }
                    } else {
                        // File were removed
                        trace!("Delete file: {}", entry.path);
                        diesel::delete(f::files.filter(f::id.eq(file.id)))
                            .execute(&mut connection)
                            .expect("Failed to delete entry");
                    }
                }
            }

            // Entry is new, insert all files
            for file in dir.files.iter() {
                if !file_lookup.contains(&file.path) {
                    trace!("Insert file: {}", file.path);
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
            .execute(&mut connection)
            .expect("Failed to execute file insert query");

        self.load_files(&mut connection);

        // Done!
        info!(
            "Found {:?} dirs and {:?} collisions. Diff: {}",
            dir_hash.len(),
            collisions.len(),
            self.entriesCache.len()
        );

        info!("Update took: {:?} ms", start.elapsed().whole_milliseconds());
    }

    pub fn get_all_entries(&self) -> &Vec<Entry> {
        return &self.entriesCache;
    }

    pub fn get_files(&self, entry: &Entry) -> Option<&Vec<File>> {
        return self.filesCache.get(&entry.id);
    }

    /*** Labels ***/
    pub fn add_entry_labels(&mut self, entry_ids: Vec<i32>, label_ids: Vec<i32>) {
        use diesel::result::Error;

        let mut insert_query = Vec::with_capacity(entry_ids.len() * label_ids.len());

        for entry_id in entry_ids.iter() {
            let map = self.entryLabelLookup.get(entry_id);

            // Add new labels
            for label_id in label_ids.iter() {
                if let Some(map) = map {
                    if map.contains(label_id) {
                        // Already in db, skip
                        continue;
                    }
                }

                insert_query.push((e2l::entry_id.eq(entry_id), e2l::label_id.eq(label_id)));
            }
        }

        let mut connection = self.establish_connection();
        connection
            .transaction::<_, Error, _>(|conn| {
                debug!("Add labels");

                for slice in insert_query.iter().collect::<Vec<_>>().chunks(5000) {
                    diesel::insert_into(e2l::entry2labels)
                        .values(slice.to_vec())
                        .execute(conn)?;
                }

                Ok(())
            })
            .expect("Failed to add_entry_labels");

        debug!("add_entry_labels() All labels done");
        self.load_labels(&mut connection);
    }

    pub fn remove_entry_labels(&mut self, entry_ids: Vec<i32>, label_ids: Vec<i32>) {
        use diesel::result::Error;

        let mut connection = self.establish_connection();
        connection
            .transaction::<_, Error, _>(|conn| {
                // Remove labels not set
                for entry_id in entry_ids.iter() {
                    diesel::delete(
                        e2l::entry2labels
                            .filter(e2l::entry_id.eq(entry_id))
                            .filter(e2l::label_id.eq_any(&label_ids)),
                    )
                    .execute(conn)?;
                }
                println!("Removed labels!");
                debug!("Labels deleted");

                Ok(())
            })
            .expect("Failed to remvove_entry_label");

        debug!("Label done");
        self.load_labels(&mut connection);
    }

    pub fn entry_labels(&self, entry_id: i32) -> Option<&HashSet<i32>> {
        return self.entryLabelLookup.get(&entry_id);
    }

    pub fn dir_labels(&self, entry_id: i32) -> Vec<i32> {
        let mut labels = Vec::new();

        for (label_id, entries) in self.labelLookupCache.iter() {
            if entries.contains(&entry_id) {
                labels.push(*label_id);
            }
        }

        return labels;
    }

    pub fn has_label(&self, entry_id: i32, label_id: i32) -> bool {
        if let Some(entries) = self.labelLookupCache.get(&label_id) {
            return entries.contains(&entry_id);
        }
        return false;
    }

    pub fn add_label(&mut self, name: &str) -> bool {
        if self.labelsCache.iter().any(|lbl| &lbl.name == name) {
            return false;
        }

        let mut connection = self.establish_connection();
        diesel::insert_into(l::labels)
            .values(l::name.eq(name))
            .execute(&mut connection)
            .expect("Failed to insert new label");

        self.labelsCache = l::labels
            .load(&mut connection)
            .expect("Failed to load labels");
        self.load_labels(&mut connection);

        return true;
    }

    pub fn remove_label(&mut self, id: i32) {
        let mut connection = self.establish_connection();

        diesel::delete(l::labels.filter(l::id.eq(id)))
            .execute(&mut connection)
            .expect("Failed to delete label");

        self.labelsCache = l::labels
            .load(&mut connection)
            .expect("Failed to load labels");
        self.load_labels(&mut connection);
    }

    pub fn get_all_labels(&self) -> &Vec<Label> {
        return &self.labelsCache;
    }

    /*** Locations ***/
    pub fn add_location(&mut self, name: &str, path: &str) {
        let mut connection = self.establish_connection();
        diesel::insert_into(loc::locations)
            .values((loc::name.eq(name), loc::path.eq(path), loc::size.eq(0)))
            .execute(&mut connection)
            .expect("Failed to insert new location");
    }

    pub fn remove_location(&mut self, id: i32) {
        let mut connection = self.establish_connection();

        diesel::delete(loc::locations.filter(loc::id.eq(id)))
            .execute(&mut connection)
            .expect("Failed to delete location");
    }

    pub fn get_locations(&self) -> Vec<Location> {
        let mut connection = self.establish_connection();

        let locations = loc::locations
            .load(&mut connection)
            .expect("Failed to load locations");
        return locations;
    }

    pub fn move_file_to_dir(&mut self, entry: &Entry, new_entry_name: &str, new_path: &str) {
        let mut connection = self.establish_connection();

        let path = Path::new(&new_path);
        let new_name = path.file_name().unwrap().to_str().unwrap();

        let entry_path = path.parent().unwrap().to_string_lossy();

        // Update file
        let file = self
            .get_files(&entry)
            .expect("Failed to find file when renaming entry")
            .iter()
            .next()
            .expect("Failed to find file when renaming entry");

        diesel::update(file)
            .set((f::name.eq(new_name), f::path.eq(new_path)))
            .execute(&mut connection)
            .expect("Failed to update path of file");

        // Update entry
        diesel::update(entry)
            .set((e::name.eq(new_entry_name), e::path.eq(entry_path)))
            .execute(&mut connection)
            .expect("Failed to update name of entry");

        self.load_from_store();
    }

    pub fn rename_entry(
        &mut self,
        entry: Entry,
        new_entry_name: &str,
        new_path: &str,
        is_file_entry: bool,
    ) {
        let mut connection = self.establish_connection();

        let path = Path::new(&new_path);
        let new_name = path.file_name().unwrap().to_str().unwrap();

        let mut files = self
            .get_files(&entry)
            .expect("Failed to find file when renaming entry")
            .into_iter();

        // Update file
        if !is_file_entry {
            // Entry is a folder, move all files in folder.

            for file in files {
                let mut path = PathBuf::from(&new_path);
                path.push(&file.name);

                println!(
                    "Update path of file: {:?} to {:?}",
                    file.name,
                    path.to_string_lossy()
                );
                diesel::update(file)
                    .set(f::path.eq(path.to_string_lossy()))
                    .execute(&mut connection)
                    .expect("Failed to update path of file");
            }
        } else {
            // Entry is just a file, change file paths and name
            let file = files.next().expect("Failed to find file for entry");
            diesel::update(file)
                .set((f::name.eq(new_entry_name), f::path.eq(new_path)))
                .execute(&mut connection)
                .expect("Failed to update path of file");
        }

        // Update entry
        diesel::update(&entry)
            .set((e::name.eq(new_entry_name), e::path.eq(new_path)))
            .execute(&mut connection)
            .expect("Failed to update name of entry");

        self.load_from_store();
    }

    pub fn set_grade(&mut self, entry: Entry, grade: i32) {
        let mut connection = self.establish_connection();

        // Update entry
        diesel::update(&entry)
            .set(e::grade.eq(grade))
            .execute(&mut connection)
            .expect("Failed to update name of entry");

        self.load_from_store();
    }

    pub fn remove_entry(&mut self, id: i32) {
        let mut connection = self.establish_connection();

        diesel::delete(e::entries.filter(e::id.eq(id)))
            .execute(&mut connection)
            .expect("Failed to delete entry");

        self.load_from_store();
        self.load_labels(&mut connection);
    }

    pub fn remove_file(&mut self, id: i32) {
        let mut connection = self.establish_connection();

        diesel::delete(f::files.filter(f::id.eq(id)))
            .execute(&mut connection)
            .expect("Failed to delete file");

        self.load_from_store();
        self.load_labels(&mut connection);
    }

    pub fn get_label_filters(&self) -> Vec<LabelAutoFilter> {
        let mut connection = self.establish_connection();

        let filters: Vec<LabelAutoFilter> = aut::label_auto_filters
            .load(&mut connection)
            .expect("Failed to load label filters");

        debug!("Got {} auto filters", filters.len());
        filters
    }

    // *** Label filters ***

    pub fn add_update_label_filter(&mut self, filter: &LabelAutoFilter) {
        use std::convert::From;
        let mut connection = self.establish_connection();

        if filter.id > 0 {
            // Update
            diesel::update(aut::label_auto_filters)
                .set(filter)
                .execute(&mut connection)
                .expect("Failed to update label filter");
        } else {
            // Add
            let insertable = LabelAutoFilterInsert::new(filter);
            diesel::insert_into(aut::label_auto_filters)
                .values(insertable)
                .execute(&mut connection)
                .expect("Failed to insert new label filter");
        }
    }

    pub fn delete_label_filter(&mut self, filter: &LabelAutoFilter) {
        use std::convert::From;
        let mut connection = self.establish_connection();

        // Delete
        diesel::delete(aut::label_auto_filters.filter(aut::id.eq(filter.id)))
            .execute(&mut connection)
            .expect("Failed to delete label filter");
    }
}
