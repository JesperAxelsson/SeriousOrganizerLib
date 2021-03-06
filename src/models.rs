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

        $(impl Into<i32> for $t {
            fn into(self) -> i32 {
                let $t(id) = self;
                id
            }
        })*
    )
}

serialize_id!(LocationId, EntryId, LabelId);

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord, Identifiable, Queryable, AsChangeset,Clone, Debug)]
#[table_name = "locations"]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct DirEntry {
    pub name: String,
    pub location_id: LocationId,
    pub path: String,
    pub files: Vec<FileEntry>,
    pub size: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Identifiable, Queryable, AsChangeset, Clone, Debug)]
#[table_name = "entries"]
pub struct Entry {
    pub id: EntryId,
    pub location_id: LocationId,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Identifiable, Queryable, AsChangeset, Clone, Debug)]
#[table_name = "files"]
pub struct File {
    pub id: i32,
    pub entry_id: EntryId,
    pub name: String,
    pub path: String,
    pub size: i64,
}

#[derive(Serialize, Identifiable, Queryable, AsChangeset, Clone, Debug)]
#[table_name = "labels"]
pub struct Label {
    pub id: LabelId,
    pub name: String,
}

#[derive(Queryable, Clone, Copy, Debug)]
pub struct Entry2Label {
    pub entry_id: EntryId,
    pub label_id: LabelId,
}
