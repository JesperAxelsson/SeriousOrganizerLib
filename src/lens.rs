// mod filter;
// extern crate intmap;

use regex::{escape, Regex, RegexBuilder};
use time::PreciseTime;

//use std::mem;
//use std::cmp::Ordering;

//use intmap::IntMap;
use models::{DirEntry, FileEntry};

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

// ************** Constant HWNDS **************

pub struct Lens {
    pub source: Vec<DirEntry>,
    pub ix_list: Vec<usize>,

    search: Search,
    //    orderby_func: Option<Box<Fn(&DirEntry)->()>>,
}

impl Lens {
    pub fn new() -> Self {
        let search = Search {
            string: String::new(),
            regex: Regex::new(".*").unwrap(),
        };

        Lens {
            source: Vec::new(),
            ix_list: Vec::new(),
            search,
            //            orderby_func: None,
        }
    }

    pub fn update_data(&mut self, mut data: &mut Vec<DirEntry>) {
        println!("Starting data update");

        self.ix_list.clear();
        self.source.clear();

        self.source.append(&mut data);

        println!("Data updated");

        self.update_ix_list();
    }

    pub fn update_ix_list(&mut self) {
        let start = PreciseTime::now();

        self.ix_list.clear();

        let search = &self.search;

        for (i, e) in self.source.iter().enumerate() {
            if search.regex.is_match(&e.name) {
                self.ix_list.push(i);
            }
        }

        let end = PreciseTime::now();

        println!(
            "ix_list update with {:?} entries took: {:?} ms",
            self.ix_list.len(),
            start.to(end).num_milliseconds()
        );
    }

    pub fn order_by<F, T>(&mut self, compare: F)
    where
        F: FnMut(&DirEntry) -> T,
        T: Ord,
    {
        self.source.sort_by_key(compare);
        self.update_ix_list();
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

    pub fn get_dir_entry(&self, ix: usize) -> Option<&DirEntry> {
        if let Some(cix) = self.convert_ix(ix) {
            return self.source.get(cix);
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
        if let Some(ref dir) = self.get_dir_entry(ix) {
            Some(dir.files.len())
        } else {
            None
        }
    }

    pub fn get_file_entry(&self, dir_ix: usize, file_ix: usize) -> Option<&FileEntry> {
        if let Some(ref dir) = self.get_dir_entry(dir_ix) {
            return dir.files.get(file_ix);
        }
        None
    }
}
