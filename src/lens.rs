#![allow(unused_imports)]
// mod filter;
// extern crate intmap;
use log::{debug, error, info, trace, warn};

use num_derive::{FromPrimitive, ToPrimitive};

use regex::{escape, Regex, RegexBuilder};
use time::Instant;

use std::collections::HashSet;
use std::fs::rename;
use std::path::Path;
use std::usize;
//use std::mem;
//use std::cmp::Ordering;

use std::fs;
use std::fs::metadata;

//use intmap::IntMap;
use crate::models::{DirEntry, Entry, EntryId, File, LabelId, Location, LocationId};
use crate::store::Store;

#[derive(Debug, Copy, Clone)]
pub enum LabelState {
    Unset,
    Exclude,
    Include,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub id: LabelId,
    pub name: String,
    pub state: LabelState,
}

pub fn create_match_regex(needle: &str) -> Regex {
    let mut res: String = String::new();

    for s in escape(needle).split_whitespace() {
        res.push_str(s);
        res.push_str(".*");
    }

    let mut build = RegexBuilder::new(&res);
    build.case_insensitive(true).unicode(true).build().unwrap()
}

const KB: u64 = 1000;
const MB: u64 = KB * KB;
const GB: u64 = KB * KB * KB;

pub fn pretty_size(size: u64) -> String {
    match size {
        x if x > GB => String::from((size / GB).to_string() + " GB"),
        x if x > MB => String::from((size / MB).to_string() + " MB"),
        x if x > KB => String::from((size / KB).to_string() + " KB"),
        _ => String::from(size.to_string() + " B"),
    }
}

#[derive(Debug)]
struct Search {
    string: String,
    regex: Regex,
}

#[derive(Debug, Clone, Copy)]
pub struct Sort {
    pub column: SortColumn,
    pub order: SortOrder,
}

impl Sort {
    pub fn new(column: SortColumn, order: SortOrder) -> Self {
        Sort { column, order }
    }
}

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, Eq, PartialEq)]
#[repr(u32)]
pub enum SortColumn {
    Name = 0,
    Path = 1,
    Date = 2,
    Size = 3,
}

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, Eq, PartialEq)]
#[repr(u32)]
pub enum SortOrder {
    Asc = 0,
    Desc = 1,
}

// ************** Constant HWNDS **************

pub struct Lens {
    pub source: Store,
    pub ix_list: Vec<usize>,
    include_labels: HashSet<LabelId>,
    exlude_labels: HashSet<LabelId>,

    /// Used for application using Lens
    label_states: Vec<Label>,

    search: Search,
    sort: Sort,
}

impl Lens {
    pub fn new() -> Self {
        let search = Search {
            string: String::new(),
            regex: Regex::new(".*").unwrap(),
        };

        let db_path = ::std::env::current_exe()
            .unwrap()
            .with_file_name("test.sqlite3");

        let mut source = Store::init(db_path.to_str().unwrap());
        source.load_from_store();

        let mut lens = Lens {
            source,
            ix_list: Vec::new(),
            search,
            sort: Sort::new(SortColumn::Name, SortOrder::Asc),

            include_labels: HashSet::new(),
            exlude_labels: HashSet::new(),

            label_states: Vec::new(),
        };
        lens.update_ix_list();

        lens
    }

    pub fn update_data(&mut self, data: &mut Vec<(LocationId, DirEntry)>) {
        trace!("Starting data update");

        self.ix_list.clear();
        self.source.update(data);

        trace!("Data updated");

        self.update_ix_list();
    }

    pub fn update_ix_list(&mut self) {
        let start = Instant::now();

        self.ix_list.clear();

        {
            let search = &self.search;

            for (i, e) in self.source.get_all_entries().iter().enumerate() {
                if search.regex.is_match(&e.name) && self.label_filter(e.id) {
                    self.ix_list.push(i);
                }
            }
        }

        self.sort();

        info!(
            "ix_list update with {:?} entries took: {:?} ms",
            self.ix_list.len(),
            start.elapsed().whole_milliseconds()
        );

        trace!("ix_list include: {:?}  ", self.include_labels);
        trace!("ix_list exclude: {:?}  ", self.exlude_labels);
    }

    fn label_filter(&self, entry_id: EntryId) -> bool {
        if self.exlude_labels.is_empty() && self.include_labels.is_empty() {
            return true;
        }

        if let Some(entry_labels) = self.source.entry_labels(entry_id) {
            if self
                .exlude_labels
                .iter()
                .any(|lbl| entry_labels.contains(lbl))
            {
                return false;
            }

            if self
                .include_labels
                .iter()
                .any(|lbl| entry_labels.contains(lbl))
            {
                return true;
            }
        }

        if self.include_labels.is_empty() {
            return true;
        } else {
            return false;
        }
    }

    pub fn order_by(&mut self, column: SortColumn, order: SortOrder) {
        self.sort = Sort::new(column, order);
        self.sort();
    }

    pub fn sort(&mut self) {
        let column = &self.sort.column;
        let order = &self.sort.order;

        let entries: &Vec<Entry> = self.source.entriesCache.as_ref();

        let selector = |ax: usize, bx: usize| {
            let a = &entries[ax];
            let b = &entries[bx];

            match column {
                SortColumn::Date => a.name.cmp(&b.name),
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Path => a.path.cmp(&b.path),
                SortColumn::Size => a.size.cmp(&b.size),
            }
        };

        self.ix_list.sort_by(move |a, b| {
            let ordered = selector(*a, *b);

            match order {
                SortOrder::Asc => ordered,
                SortOrder::Desc => ordered.reverse(),
            }
        });
    }

    pub fn update_search_text(&mut self, new_string: &str) -> Option<usize> {
        if new_string != self.search.string {
            self.search.regex = create_match_regex(new_string);
            self.search.string = String::from(new_string);

            self.update_ix_list();
            return Some(self.ix_list.len());
        }

        None
    }

    // *** Entries ***

    pub fn get_dir_count(&self) -> usize {
        self.ix_list.len()
    }

    pub fn get_dir_entry(&self, ix: usize) -> Option<&Entry> {
        if let Some(cix) = self.convert_ix(ix) {
            return self.source.entriesCache.get(cix);
        }
        None
    }

    pub fn convert_ix(&self, ix: usize) -> Option<usize> {
        if ix < self.ix_list.len() {
            Some(self.ix_list[ix])
        } else {
            None
        }
    }

    // *** Files ***

    pub fn get_dir_files(&self, ix: usize) -> Option<&Vec<File>> {
        if let Some(entry) = self.get_dir_entry(ix) {
            return self.source.get_files(entry);
        }
        None
    }

    pub fn get_file_count(&self, ix: usize) -> Option<usize> {
        if let Some(ref files) = self.get_dir_files(ix) {
            Some(files.len())
        } else {
            None
        }
    }

    pub fn get_file_entry(&self, dir_ix: usize, file_ix: usize) -> Option<&File> {
        if let Some(ref files) = self.get_dir_files(dir_ix) {
            return files.get(file_ix);
        }
        None
    }

    // *** Labels ***

    pub fn add_inlude_label(&mut self, label_id: u32) {
        let label_id = LabelId(label_id as i32);
        self.exlude_labels.remove(&label_id);
        self.include_labels.insert(label_id);

        self.update_ix_list();
        self.update_label_states();
    }

    pub fn add_exclude_label(&mut self, label_id: u32) {
        let label_id = LabelId(label_id as i32);
        self.include_labels.remove(&label_id);
        self.exlude_labels.insert(label_id);
        self.update_ix_list();

        self.update_label_states();
    }

    pub fn remove_label_filter(&mut self, label_id: u32) {
        let label_id = LabelId(label_id as i32);
        self.exlude_labels.remove(&label_id);
        self.include_labels.remove(&label_id);
        self.update_ix_list();

        self.update_label_states();
    }

    pub fn add_label(&mut self, name: &str) {
        self.source.add_label(name);

        self.update_label_states();
    }

    pub fn remove_label(&mut self, label_id: u32) {
        self.source.remove_label(LabelId(label_id as i32));
        self.remove_label_filter(label_id);

        self.update_label_states();
    }

    pub fn update_label_states(&mut self) {
        let labels = self.source.get_all_labels();
        self.label_states.clear();

        for lbl in labels {
            let lbl_state = if self.include_labels.contains(&lbl.id) {
                LabelState::Include
            } else if self.exlude_labels.contains(&lbl.id) {
                LabelState::Exclude
            } else {
                LabelState::Unset
            };

            self.label_states.push(Label {
                id: lbl.id.clone(),
                name: lbl.name.clone(),
                state: lbl_state,
            });
        }
    }

    pub fn get_labels(&self) -> &Vec<Label> {
        &self.label_states
    }

    pub fn entry_labels(&self, id: u32) -> Vec<LabelId> {
        self.source.dir_labels(EntryId(id as i32))
    }

    pub fn add_entry_labels(&mut self, entries: Vec<u32>, labels: Vec<u32>) {
        let start = Instant::now();
        let count = entries.len();
        self.source.add_entry_labels(
            entries.into_iter().map(|e| EntryId(e as i32)).collect(),
            labels.into_iter().map(|l| LabelId(l as i32)).collect(),
        );

        trace!(
            "set_entry_labels update with {:?} entries took: {:?} ms",
            count,
            start.elapsed().whole_milliseconds()
        );
    }

    pub fn remove_entry_labels(&mut self, entries: Vec<u32>, labels: Vec<u32>) {
        let start = Instant::now();
        let count = entries.len();
        self.source.remove_entry_labels(
            entries.into_iter().map(|e| EntryId(e as i32)).collect(),
            labels.into_iter().map(|l| LabelId(l as i32)).collect(),
        );

        trace!(
            "set_entry_labels update with {:?} entries took: {:?} ms",
            count,
            start.elapsed().whole_milliseconds()
        );
    }

    /*** Locations ***/
    pub fn add_location(&mut self, name: &str, path: &str) {
        self.source.add_location(name, path);
    }

    pub fn remove_location(&mut self, id: u32) {
        self.source.remove_location(LocationId(id as i32));
    }

    pub fn remove_location_id(&mut self, id: LocationId) {
        self.source.remove_location(id);
    }

    pub fn get_locations(&self) -> Vec<Location> {
        self.source.get_locations()
    }

    /*** Rename ***/
    pub fn rename_entry(&mut self, entry: Entry, new_name: &str) -> bool {
        let new_meta = metadata(new_name);

        if new_meta.is_ok() {
            // Path already exists
            return false;
        }

        let old_meta = metadata(&entry.path);

        if old_meta.is_err() {
            return false;
        }
        
        let path = Path::new(&entry.path);
        let new_path = path.with_file_name(new_name);

        rename(&entry.path, new_path).expect("Failed to rename file");

        self.source.rename_entry(entry, new_name);

        self.source.load_from_store();
        self.update_ix_list();

        true
    }
}
