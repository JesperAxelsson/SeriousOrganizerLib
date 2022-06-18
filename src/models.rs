#![allow(proc_macro_derive_resolution_fallback)]

use crate::schema::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Identifiable, Queryable, AsChangeset, Clone, Debug)]
#[table_name = "locations"]
pub struct Location {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Clone, Debug)]
pub struct DirEntry {
    pub name: String,
    pub location_id: i32,
    pub path: String,
    pub files: Vec<FileEntry>,
    pub size: u64,
}

#[derive(Clone, Debug, Ord, PartialEq, Eq, PartialOrd)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Identifiable, Queryable, AsChangeset, Clone, Debug)]
#[table_name = "entries"]
pub struct Entry {
    pub id: i32,
    pub location_id: i32,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Identifiable, Queryable, AsChangeset, Clone, Debug)]
#[table_name = "files"]
pub struct File {
    pub id: i32,
    pub entry_id: i32,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Identifiable, Queryable, AsChangeset, Clone, Debug)]
#[table_name = "labels"]
pub struct Label {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Clone, Copy, Debug)]
pub struct Entry2Label {
    pub entry_id: i32,
    pub label_id: i32,
}

#[derive(Identifiable, Queryable, AsChangeset, Clone, Debug)]
#[table_name = "label_auto_filters"]
pub struct LabelAutoFilter {
    pub id: i32,
    pub name: String,
    pub filter: String,
    pub label_id: i32,
}

#[derive(Insertable, AsChangeset, Clone, Debug)]
#[table_name = "label_auto_filters"]
pub(crate) struct LabelAutoFilterInsert {
    pub name: String,
    pub filter: String,
    pub label_id: i32,
}

impl LabelAutoFilterInsert {
    pub fn new(item: &LabelAutoFilter) -> Self {
        LabelAutoFilterInsert {
            name: item.name.clone(),
            filter: item.filter.clone(),
            label_id: item.label_id,
        }
    }
}
