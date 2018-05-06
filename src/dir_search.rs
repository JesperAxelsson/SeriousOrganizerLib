use std::fs;
use std::path::PathBuf;
use std::mem;
use std::cmp::Ordering;
use std::thread;

use time::PreciseTime;


use scan_dir::ScanDir;


#[derive(Serialize, Deserialize, Debug)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub files: Vec<FileEntry>,
    pub size: u64,
}


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


#[derive(Serialize, Deserialize, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
}


pub fn list_files_in_dir(path: &str) -> Vec<DirEntry> {
    // println!("Starting glob: {:?}", path);

    let mut vec: Vec<DirEntry> = Vec::new();

    let path = PathBuf::from(path);

    if !path.exists() {
        println!("Sorry path: {:?} does not exits", path);
        return vec;
    }

    let sized = |files: &Vec<FileEntry>| {
        let mut size = 0;
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
                    let ff =
                        vec![FileEntry {
                            name: path.file_name().unwrap().to_str().unwrap().to_string(),
                            path: path.to_str().unwrap().to_string(),
                            size: meta.len(),
                        }];

                    let e = DirEntry {
                        name: path.file_name().unwrap().to_str().unwrap().to_string(),
                        path: path.to_str().unwrap().to_string(),
                        files: ff,
                        size: meta.len(),
                    };

                    vec.push(e);
                }

                // *** Handle dir ***
                if meta.is_dir() {
                    let mut files = Vec::new();

                    // println!("***");

                    ScanDir::files()
                        .walk(&path, |iter| {
                            for (entry, name) in iter {
                                // println!("File {:?} has full path {:?}", name, entry.path());
                                let dir = entry.path();

                                // println!("File: {:?}", &dir);

                                let dir_path = dir.to_str().expect("Failed to read path");

                                let size = if dir.to_str().unwrap().len() >= 260 {
                                    let strr = "\\??\\".to_owned() + dir.to_str().unwrap();
                                    fs::metadata(&strr).expect("failed to read metadata").len()
                                } else {
                                    fs::metadata(&dir).expect("failed to read metadata").len()
                                };

                                let dir = dir_path.to_string();
                                // let name = shared_path(&dir, name.len());;
                                files.push(FileEntry {
                                    // name: String::from(name),
                                    name: name,
                                    // path: String::from(dir_path),
                                    path: dir,
                                    size: size,
                                });
                            }
                        })
                        .unwrap();


                    let size = sized(&files);
                    // files.shrink_to_fit();

                    // let len = path.file_name().unwrap().to_str().unwrap().len();

                    let name = path.file_name().unwrap().to_str().unwrap().to_string();
                    let path = path.to_str().unwrap().to_string();

                    // let name = shared_path(&path, len);


                    let e = DirEntry {
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


pub fn get_all_data(paths: Vec<String>) -> Vec<DirEntry> {
    let mut vec = Vec::new();

    let start = PreciseTime::now();

    let mut children = Vec::new();

    for p in paths {
        children.push(thread::spawn(move || {
            let start = PreciseTime::now();

            let vec1 = list_files_in_dir(&p);

            let end = PreciseTime::now();

            println!("Path {:?} entries took: {:?} ms",
                     &p,
                     start.to(end).num_milliseconds());
            vec1
        }))
    }

    for c in children {
        vec.append(&mut c.join().expect("Failed to join thread!"));
    }

    vec.sort();

    let end = PreciseTime::now();

    println!("Got {:?} entries took: {:?} ms",
             vec.len(),
             start.to(end).num_milliseconds());

    vec
}
