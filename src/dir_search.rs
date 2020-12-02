#![allow(unused_imports)]
use log::{error, info, trace, warn};
use std::cmp::Ordering;
use std::fs;
use std::mem;
use std::path::PathBuf;
use std::thread;

use time::Instant;

use crate::models::*;
use scan_dir::ScanDir;

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

impl DirEntry {
    pub fn get_file_entry(&self, ix: usize) -> Option<&FileEntry> {
        if ix < self.files.len() {
            Some(&self.files[ix])
        } else {
            None
        }
    }
}

pub fn list_files_in_dir(location_id: LocationId, path: &str) -> Vec<DirEntry> {
    trace!("Starting glob: {:?}", path);

    let mut vec: Vec<DirEntry> = Vec::new();

    let path = PathBuf::from(path);

    if !path.exists() {
        info!("Sorry path: {:?} does not exits", path);
        return vec;
    }

    let sized = |files: &Vec<FileEntry>| {
        let mut size: u64 = 0;
        for f in files.iter() {
            size += f.size;
        }
        size
    };

    ScanDir::all()
        .skip_hidden(true)
        .read(path, |iter| {
            for (path, _) in iter {
                let path = path.path();
                let meta = path.metadata().expect("Failed to read metadata");

                // *** Handle file ***
                if meta.is_file() {
                    let ff = vec![FileEntry {
                        name: path.file_name().expect("Failed to read filename").to_str().expect("Failed to convert filename to rust string").to_string(),
                        path: path.to_str().expect("Failed to convert filepath to rust string").to_string(),
                        size: meta.len(),
                    }];

                    let e = DirEntry {
                        location_id,
                        name: path.file_name().expect("Failed to read dirname").to_str().expect("Failed to convert dirname to rust string").to_string(),
                        path: path.to_str().expect("Failed to convert dirpath to rust string").to_string(),
                        files: ff,
                        size: meta.len(),
                    };

                    vec.push(e);
                }

                // *** Handle dir ***
                if meta.is_dir() {
                    let mut files = Vec::new();

                    trace!("***");

                    ScanDir::files()
                        .walk(&path, |iter| {
                            for (entry, name) in iter {
                                trace!("File {:?} has full path {:?}", name, entry.path());
                                let dir = entry.path();

                                trace!("File: {:?}", &dir);

                                let size = get_size(&dir);

                                // let name = shared_path(&dir, name.len());;
                                files.push(FileEntry {
                                    name: name,
                                    path: dir.to_string_lossy().into(),
                                    size: size,
                                });
                            }
                        })
                        .expect("Failed to run ScanDir::Files()");

                    let size = sized(&files);
                    // files.shrink_to_fit();

                    // let len = path.file_name().unwrap().to_str().unwrap().len();

                    let name = path.file_name().expect("Failed to read dirname").to_str().expect("Failed to convert dirname to rust string").to_string();
                    let path = path.to_str().expect("Failed to convert dirpath to rust string").to_string();

                    // let name = shared_path(&path, len);

                    let e = DirEntry {
                        location_id,
                        name: name,
                        path: path,
                        files: files,
                        size: size,
                    };

                    vec.push(e);
                }
            }
        })
        .expect("Scan dir failed!");

    vec
}

fn get_size(dir_path: &PathBuf) -> u64 {
    if cfg!(windows) {
        let dir_path = dir_path.to_str().expect("Failed to read path");

        if dir_path.len() >= 260 {
            let strr = "\\??\\".to_owned() + dir_path;
            fs::metadata(&strr).expect("failed to read metadata").len()
        } else {
            fs::metadata(&dir_path).expect("failed to read metadata").len()
        }     
    } else {
        fs::metadata(&dir_path).expect("failed to read metadata").len()
    }
}


pub fn get_all_data(paths: Vec<(LocationId, String)>) -> Vec<(LocationId, DirEntry)> {
    let mut vec = Vec::new();

    let start =Instant::now();

    let mut children = Vec::new();

    for p in paths {
        children.push(thread::spawn(move || {
            let start = Instant::now();

            let vec1 = list_files_in_dir(p.0, &p.1).into_iter().map(|d| (p.0, d)).collect();

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
        "Got {:?} entries took: {:?} ms",
        vec.len(),
        start.elapsed().whole_milliseconds()
    );

    vec
}
