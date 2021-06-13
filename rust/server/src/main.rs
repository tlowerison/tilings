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

fn set_env() {
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
    std::env::set_var("ROCKET_DATABASES", format!("{{pg_db={{url=\"{}\"}}}}", database_url));
}

#[launch]
fn rocket() -> _ {
    set_env();

    rocket::build().mount("/", routes![
        api::get_tiling,
        api::create_tiling,
        api::update_tiling,
        api::delete_tiling,
        api::match_labels,
        api::add_label_to_tiling,
    ]).attach(connection::DbConn::fairing())
}
