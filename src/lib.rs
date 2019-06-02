#![allow(non_snake_case)]
#![allow(proc_macro_derive_resolution_fallback)]


//extern crate walkdir;
//extern crate rusqlite;

#[macro_use]
extern crate serde_derive;

extern crate num_derive;


#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_newtype;
#[macro_use]
extern crate diesel_migrations;

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
