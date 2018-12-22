// mod filter;
// extern crate intmap;

use num_derive::{FromPrimitive, ToPrimitive};

use regex::{escape, Regex, RegexBuilder};
use time::PreciseTime;

//use std::mem;
//use std::cmp::Ordering;

//use intmap::IntMap;
use crate::models::{DirEntry, Entry, File};
use crate::store::Store;


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

#[derive(Debug)]
pub struct Sort {
    column: SortColumn,
    order: SortOrder,
}

impl Sort {
    pub fn new(column: SortColumn, order: SortOrder) -> Self {
        Sort { column, order }
    }
}


#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum SortColumn {
    Name = 0,
    Path = 1,
    Date = 2,
    Size = 3,
}

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum SortOrder {
    Asc = 0,
    Desc = 1,
}


// ************** Constant HWNDS **************

pub struct Lens {
    pub source: Store,
    pub ix_list: Vec<usize>,

    search: Search,
    sort: Sort,
}

impl Lens {
    pub fn new() -> Self {
        let search = Search {
            string: String::new(),
            regex: Regex::new(".*").unwrap(),
        };

        let mut source = Store::init();
        source.load_from_store();

        Lens {
            source,
            ix_list: Vec::new(),
            search,
            sort: Sort::new(SortColumn::Name, SortOrder::Asc),
        }
    }

    pub fn update_data(&mut self, data: &mut Vec<DirEntry>) {
        println!("Starting data update");

        self.ix_list.clear();
        self.source.update(data);

        println!("Data updated");

        self.update_ix_list();
    }

    pub fn update_ix_list(&mut self) {
        let start = PreciseTime::now();

        self.ix_list.clear();

        {
            let search = &self.search;

            for (i, e) in self.source.get_all_entries().iter().enumerate() {
                if search.regex.is_match(&e.name) {
                    self.ix_list.push(i);
                }
            }
        }

        self.sort();

        let end = PreciseTime::now();

        println!(
            "ix_list update with {:?} entries took: {:?} ms",
            self.ix_list.len(),
            start.to(end).num_milliseconds()
        );
    }

    pub fn order_by(&mut self, column: SortColumn, order: SortOrder)
    {
        self.sort = Sort::new(column, order);
        self.sort();
    }


    pub fn sort(&mut self) {
        let column = &self.sort.column;
        let order = self.sort.order;

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

    pub fn get_dir_entry(&self, ix: usize) -> Option<&Entry> {
        if let Some(cix) = self.convert_ix(ix) {
            return self.source.entriesCache.get(cix);
        }
        None
    }

    pub fn get_dir_files(&self, ix: usize) -> Option<&Vec<File>> {
        if let Some(entry) = self.get_dir_entry(ix) {
            return self.source.get_files(entry);
        }
        None
    }

    fn convert_ix(&self, ix: usize) -> Option<usize> {
        if ix < self.ix_list.len() {
            Some(self.ix_list[ix])
        } else {
            None
        }
    }

    pub fn get_dir_count(&self) -> usize {
        self.ix_list.len()
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
}
