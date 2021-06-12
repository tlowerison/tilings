#![recursion_limit="512"]

pub mod api;
pub mod connection;
pub mod models;
pub mod queries;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate mashup;
extern crate rocket;

use rocket::{launch, routes};

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![
        api::get_tiling,
        api::create_tiling,
        api::update_tiling,
        api::delete_tiling,
        api::match_labels,
        api::add_label_to_tiling,
    ]).attach(connection::DbConn::fairing())
}
