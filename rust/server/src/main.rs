#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate rocket;

use api::*;
use db_conn::DbConn;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use r2d2_redis::{r2d2, RedisConnectionManager};

embed_migrations!();

fn format_url(protocol: &str, user: Option<String>, password: Option<String>, hostname: String, port: String, dbname: Option<String>) -> String {
    let account = match user {
        None => String::from(""),
        Some(user) => format!(
            "{}{}@",
            user,
            match password {
                None => String::from(""),
                Some(password) => format!(":{}", password),
            }
        ),
    };
    format!(
        "{}://{}{}:{}{}",
        protocol,
        account,
        hostname,
        port,
        match dbname {
            Some(dbname) => format!("/{}", dbname),
            None => String::from(""),
        },
    )
}

fn set_env() -> (String, String) {
    // postgres
    // pulled from shared secret
    let postgres_user = std::env::var("POSTGRES_USER").ok();
    let postgres_password = std::env::var("POSTGRES_PASSWORD").ok();
    let postgres_dbname = std::env::var("POSTGRES_DB").unwrap();

    // set explicitly
    let postgres_service_name = std::env::var("POSTGRES_SERVICE_NAME").unwrap();

    // pulled from environment - set explicitly or with k8s service discovery
    let postgres_hostname = std::env::var(format!("{}_{}", postgres_service_name, "SERVICE_HOST")).unwrap();
    let postgres_port = std::env::var(format!("{}_{}", postgres_service_name, "SERVICE_PORT")).unwrap();

    let postgres_url = format_url(
        "postgres",
        postgres_user,
        postgres_password,
        postgres_hostname,
        postgres_port,
        Some(postgres_dbname),
    );

    std::env::set_var("DATABASE_URL", postgres_url.clone());
    std::env::set_var("ROCKET_DATABASES", format!("{{pg_db={{url=\"{}\"}}}}", postgres_url.clone()));

    // redis
    // pulled from shared secret
    let redis_user = std::env::var("REDIS_USER").ok();
    let redis_password = std::env::var("REDIS_PASSWORD").ok();
    let redis_dbname = std::env::var("REDIS_DB").ok();

    // set explicitly
    let redis_service_name = std::env::var("REDIS_SERVICE_NAME").unwrap();

    // pulled from environment - set explicitly or with k8s service discovery
    let redis_hostname = std::env::var(format!("{}_{}", redis_service_name, "SERVICE_HOST")).unwrap();
    let redis_port = std::env::var(format!("{}_{}", redis_service_name, "SERVICE_PORT")).unwrap();

    let redis_url = format_url(
        "redis",
        redis_user,
        redis_password,
        redis_hostname,
        redis_port,
        redis_dbname,
    );

    (postgres_url, redis_url)
}

#[launch]
fn rocket() -> _ {
    let (postgres_url, redis_url) = set_env();

    let postgres_conn = &PgConnection::establish(&postgres_url).expect(&format!("Error connecting to {}", postgres_url));
    embedded_migrations::run(postgres_conn).expect(&format!("Error running pending migrations"));

    let redis_manager = RedisConnectionManager::new(redis_url).unwrap();
    let redis_pool = r2d2::Pool::builder()
        .build(redis_manager)
        .unwrap();

    rocket::build()
        .manage(redis_pool)
        .mount("/", routes![
            add_label_to_polygon,
            check_email,
            check_display_name,
            create_polygon,
            delete_label,
            delete_polygon,
            get_atlas,
            get_atlases,
            get_labels,
            get_polygon,
            get_polygons,
            get_tiling,
            get_tilings,
            get_tiling_type,
            get_tiling_types,
            match_labels,
            reset_api_key,
            sign_in,
            sign_out,
            sign_up,
            text_search,
            upsert_label,
            update_polygon,
        ])
        .attach(DbConn::fairing())
}
