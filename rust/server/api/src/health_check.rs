use async_std::task;
use db_conn::DbConn;
use futures::{future::select_all, FutureExt};
use lazy_static::lazy_static;
use r2d2_redis::{r2d2::Pool, redis, RedisConnectionManager};
use result::{Error, Result};
use rocket::{serde::json::Json, State};
use std::{ops::DerefMut, time::Duration};

lazy_static!{
    // task sleep always sleeps 30 seconds regardless of input *shrug*
    static ref HEALTH_CHECK_MAX_DURATION: Duration = Duration::from_secs(30);
}

#[get("/health")]
pub async fn health(db: DbConn, redis_pool: &State<Pool<RedisConnectionManager>>) -> Result<Json<String>> {
    let (res, _, _) = select_all(vec![
        fallback_health().boxed(),
        service_health(db, redis_pool).boxed(),
    ]).await;
    res?;
    Ok(Json::from(String::from("Ok")))
}

pub async fn service_health(db: DbConn, redis_pool: &State<Pool<RedisConnectionManager>>) -> Result<()> {
    let db_health_check = db_health(db);
    let redis_health_check = redis_health(redis_pool);
    db_health_check.await?;
    redis_health_check.await?;
    Ok(())
}

pub async fn db_health(db: DbConn) -> Result<()> {
    db.run(move |_conn| Ok(()) as Result<()>)
        .await
        .map_err(Error::from)
}

pub async fn redis_health(redis_pool: &State<Pool<RedisConnectionManager>>) -> Result<()> {
    redis::cmd("PING")
       .query::<()>(redis_pool.get()?.deref_mut())
       .map_err(Error::from)
}

pub async fn fallback_health() -> Result<()> {
    task::sleep(*HEALTH_CHECK_MAX_DURATION).await;
    Err(Error::Default)
}
