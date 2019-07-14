#![allow(proc_macro_derive_resolution_fallback)]

use crate::schema::*;
use serde::ser::{Serialize, Serializer};


#[derive(PartialEq, Eq, PartialOrd, Ord, DieselNewType, Debug, Copy, Clone, Hash)]
pub struct LocationId(pub i32);

#[derive(PartialEq, Eq, PartialOrd, Ord, DieselNewType, Debug, Copy, Clone, Hash)]
pub struct EntryId(pub i32);

#[derive(PartialEq, Eq, PartialOrd, Ord, DieselNewType, Debug, Copy, Clone, Hash)]
pub struct LabelId(pub i32);

macro_rules! serialize_id {
    ($($t:tt),*) => (
        $(impl Serialize for $t {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
            {
                let $t(id) = *self;
                serializer.serialize_i32(id)
            }   
        })*
    )
}

serialize_id!(LocationId, EntryId, LabelId);


#[derive(PartialEq, Eq, PartialOrd, Ord, Identifiable, Queryable, AsChangeset, Debug)]
#[table_name = "locations"]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Serialize, Debug)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub files: Vec<FileEntry>,
    pub size: u64,
}

#[derive(Serialize, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Identifiable, Queryable, AsChangeset, Debug)]
#[table_name = "entries"]
pub struct Entry {
    pub id: EntryId,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Identifiable, Queryable, AsChangeset, Debug)]
#[table_name = "files"]
pub struct File {
    pub id: i32,
    pub entry_id: EntryId,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Serialize, Identifiable, Queryable, AsChangeset, Debug)]
#[table_name = "labels"]
pub struct Label {
    pub id: LabelId,
    pub name: String,
}

#[derive(Queryable, Debug)]
//#[table_name = "entry2labels"]
pub struct Entry2Label {
    pub entry_id: EntryId,
    pub label_id: LabelId,
}

#[derive(Queryable, Debug)]
//#[table_name = "entry2labels"]
pub struct Location2Entry {
    pub location_id: LocationId,
    pub entry_id: EntryId,
}