use diesel::{self, pg::PgConnection, r2d2::{self, ConnectionManager}};
use rocket::response::Debug;
use rocket_sync_db_pools::database;
use std::env;
use std::ops::Deref;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url());
    Pool::new(manager).expect("db pool")
}

fn database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

#[database("pg_db")]
pub struct DbConn(pub PgConnection);

impl Deref for DbConn {
    type Target = rocket_sync_db_pools::Connection<DbConn, PgConnection>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;
