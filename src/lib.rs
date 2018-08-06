#![allow(non_snake_case)]
extern crate scan_dir;
extern crate walkdir;
extern crate time;
extern crate regex;
//extern crate rusqlite;
extern crate serde;
#[macro_use] extern crate serde_derive;

#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod dir_search;
pub mod lens;
pub mod store;
pub mod schema;
pub mod models;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
