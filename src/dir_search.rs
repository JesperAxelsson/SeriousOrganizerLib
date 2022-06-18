#![allow(unused_imports)]
use log::{error, info, trace, warn};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fs;
use std::fs::Metadata;
use std::mem;
use std::path::PathBuf;
use std::thread;

use time::Instant;

use crate::models::*;
use jwalk::WalkDir;

impl Drop for DirEntry {
    fn drop(&mut self) {
        self.files.clear();
        mem::drop(&self.files);
    }
}

impl Ord for DirEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for DirEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl PartialEq for DirEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for DirEntry {}

fn list_files_in_dir(location_id: i32, path: &str) -> Vec<DirEntry> {
    // trace!("Starting glob: {:?}", path);

    let mut vec: Vec<DirEntry> = Vec::new();

    let current_dir: RefCell<Option<DirEntry>> = RefCell::new(None);

    for entry in WalkDir::new(path)
        .sort(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let meta = get_meta(&entry.path());

        // *** Handle file ***
        if meta.is_file() && entry.depth() == 1 {
            // println!("Found root file {:?} {}", entry.path(), entry.depth);
            let file_name = entry
                .file_name()
                .to_str()
                .expect("Failed to convert filename to rust string")
                .to_string();
            let path = entry
                .path()
                .to_str()
                .expect("Failed to convert filepath to rust string")
                .to_string();

            let ff = vec![FileEntry {
                name: file_name.clone(),
                path: path.clone(),
                size: meta.len(),
            }];

            let e = DirEntry {
                location_id,
                name: file_name,
                path: path,
                files: ff,
                size: meta.len(),
            };

            vec.push(e);
        } else if meta.is_file() && entry.depth() > 0 {
            // Add files to dir entry
            // println!("Found file {:?} ", entry.path());

            let path = entry
                .path()
                .to_str()
                .expect("Failed to convert filepath to rust string")
                .to_string();

            let ff = FileEntry {
                name: entry
                    .file_name()
                    .to_str()
                    .expect("Failed to convert filename to rust string")
                    .to_string(),
                path: path,
                size: meta.len(),
            };

            if let Some(dir) = &mut *current_dir.borrow_mut() {
                dir.size += ff.size;
                dir.files.push(ff);
            }
        } else
        // *** Handle dir ***
        if meta.is_dir() && entry.depth() == 1 {
            // Create new dir entry to work in
            // println!("Found root dir {:?} {}", entry.path(), entry.depth);

            let foo = current_dir.borrow_mut().take();
            if let Some(f) = foo {
                vec.push(f);
            }

            // trace!("***");

            let name = entry
                .file_name()
                .to_str()
                .expect("Failed to convert dirname to rust string")
                .to_string();
            let path = entry
                .path()
                .to_str()
                .expect("Failed to convert dirpath to rust string")
                .to_string();

            let dir = DirEntry {
                location_id,
                name: name,
                path: path,
                files: Vec::new(),
                size: 0,
            };

            *current_dir.borrow_mut() = Some(dir);
        }
    }

    let foo = current_dir.borrow_mut().take();
    if let Some(f) = foo {
        vec.push(f);
    }

    vec
}

#[inline]
fn get_meta(dir_path: &PathBuf) -> Metadata {
    if cfg!(windows) {
        let dir_path = dir_path.to_str().expect("Failed to read path");

        if dir_path.len() >= 260 {
            let strr = "\\??\\".to_owned() + dir_path;
            fs::metadata(&strr).expect("failed to read metadata")
        } else {
            fs::metadata(&dir_path).expect("failed to read metadata")
        }
    } else {
        fs::metadata(&dir_path).expect("failed to read metadata")
    }
}

pub fn get_all_data(paths: &Vec<(i32, String)>) -> Vec<(i32, DirEntry)> {
    let mut vec = Vec::new();

    let start = Instant::now();

    let mut children = Vec::new();

    for p in paths.iter().cloned() {
        children.push(thread::spawn(move || {
            let start = Instant::now();

            let vec1 = list_files_in_dir(p.0, &p.1)
                .into_iter()
                .map(|d| (p.0, d))
                .collect();

            info!(
                "Path {:?} entries took: {:?} ms",
                &p.1,
                start.elapsed().whole_milliseconds()
            );
            vec1
        }))
    }

    for c in children {
        vec.append(&mut c.join().expect("Failed to join thread!"));
    }

    vec.sort();

    info!(
        "Got {:?} entries took: {:?} ms, walkdir",
        vec.len(),
        start.elapsed().whole_milliseconds()
    );

    vec
}
