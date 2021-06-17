#![recursion_limit="1024"]

pub mod api;
pub mod connection;
pub mod models;
pub mod queries;
pub mod schema;

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate mashup;
#[macro_use] extern crate rocket;

use diesel::prelude::*;
use diesel::pg::PgConnection;

embed_migrations!();

fn get_database_url(user: String, password: String, hostname: String, port: String, dbname: String) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        user,
        password,
        hostname,
        port,
        dbname,
    )
}

// returns DATABASE_URL
fn set_env() -> String {
    // pulled from shared secret
    let user = std::env::var("POSTGRES_USER").unwrap();
    let password = std::env::var("POSTGRES_PASSWORD").unwrap();
    let dbname = std::env::var("POSTGRES_DB").unwrap();

    // set explicitly
    let db_service_name = std::env::var("DB_SERVICE_NAME").unwrap();

    // pulled from environment - set explicitly or with k8s service discovery
    let hostname = std::env::var(format!("{}_{}", db_service_name, "SERVICE_HOST")).unwrap();
    let port = std::env::var(format!("{}_{}", db_service_name, "SERVICE_PORT")).unwrap();

    let database_url = get_database_url(user, password, hostname, port, dbname);

    std::env::set_var("DATABASE_URL", database_url.clone());
    std::env::set_var("ROCKET_DATABASES", format!("{{pg_db={{url=\"{}\"}}}}", database_url.clone()));

    database_url
}

#[launch]
fn rocket() -> _ {
    let database_url = set_env();

    embedded_migrations::run(
        &PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
    ).expect(&format!("Error running pending migrations"));

    rocket::build().mount("/", routes![
        api::match_labels,
        api::upsert_label,
        api::delete_label,
        api::get_polygon,
        api::get_polygons,
        api::create_polygon,
        api::update_polygon,
        api::delete_polygon,
        api::text_search,
    ]).attach(connection::DbConn::fairing())
}
