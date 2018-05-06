#![allow(non_snake_case)]
extern crate scan_dir;
extern crate time;
extern crate regex;
extern crate rusqlite;
extern crate serde;
#[macro_use] extern crate serde_derive;

pub mod dir_search;
pub mod lens;
pub mod db;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
