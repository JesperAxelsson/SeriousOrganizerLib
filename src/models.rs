#![allow(proc_macro_derive_resolution_fallback)]

use schema::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub files: Vec<FileEntry>,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
}

#[derive(Identifiable, Queryable, AsChangeset, Debug)]
#[table_name = "entries"]
pub struct Entry {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Identifiable, Queryable, AsChangeset, Debug)]
#[table_name = "files"]
pub struct File {
    pub id: i32,
    pub entry_id: i32,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Identifiable, Queryable, AsChangeset, Debug)]
#[table_name = "labels"]
pub struct Label {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Debug)]
//#[table_name = "entry2labels"]
pub struct Entry2Label {
    pub entry_id: i32,
    pub label_id: i32,
}
