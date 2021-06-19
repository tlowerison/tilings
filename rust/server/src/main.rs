#![recursion_limit="1024"]

pub mod api;
pub mod connection;
pub mod models;
pub mod queries;
pub mod response;
pub mod result;
pub mod schema;

extern crate argon2;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate itertools;
#[macro_use] extern crate mashup;
#[macro_use] extern crate rocket;

use api::*;
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
    let conn = &PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url));
    embedded_migrations::run(conn).expect(&format!("Error running pending migrations"));

    rocket::build().mount("/", routes![
        check_email,
        check_display_name,
        create_polygon,
        delete_label,
        delete_polygon,
        get_atlas,
        get_atlases,
        get_polygon,
        get_polygons,
        get_tiling,
        get_tilings,
        get_tiling_type,
        get_tiling_types,
        match_labels,
        sign_in,
        sign_out,
        sign_up,
        text_search,
        upsert_label,
        update_polygon,
    ]).attach(connection::DbConn::fairing())
}
