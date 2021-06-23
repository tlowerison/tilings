#![recursion_limit="1024"]

#[macro_use] extern crate diesel;

mod schema;

pub use self::schema::*;
