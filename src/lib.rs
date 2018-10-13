#![allow(non_snake_case)]
#![allow(proc_macro_derive_resolution_fallback)]
extern crate regex;
extern crate scan_dir;
extern crate time;
extern crate walkdir;
//extern crate rusqlite;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod dir_search;
pub mod lens;
pub mod models;
pub mod schema;
pub mod store;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
