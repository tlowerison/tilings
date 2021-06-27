use auth::*;
use db_conn::DbConn;
use models::*;
use queries;
use r2d2_redis::{
    r2d2::{Pool, PooledConnection},
    redis,
    RedisConnectionManager,
};
use rand::{self, distributions::Alphanumeric, Rng};
use result::{Error, Result};
use rocket::{
    http::{Cookie, CookieJar},
    serde::json::Json,
    State,
};
use std::ops::DerefMut;

pub const COOKIE_LENGTH: usize = 32;

#[get("/v1/check-email/<email>")]
pub async fn check_email(email: String, db: DbConn) -> Result<Json<bool>> {
    db.run(move |conn| queries::check_email(email, conn)).await.map(Json)
}

#[get("/v1/check-display-name/<display_name>")]
pub async fn check_display_name(display_name: String, db: DbConn) -> Result<Json<bool>> {
    db.run(move |conn| queries::check_display_name(display_name, conn)).await.map(Json)
}

#[get("/v1/reset-api-key")]
pub async fn reset_api_key(db: DbConn, auth_account: AuthAccount) -> Result<Json<String>> {
    db.run(move |conn| conn.build_transaction().run(|| queries::reset_api_key(auth_account, conn))).await.map(Json)
}

#[post("/v1/sign-in", data = "<sign_in_post>")]
pub async fn sign_in(sign_in_post: SignInPost, db: DbConn, jar: &CookieJar<'_>, redis_pool: &State<Pool<RedisConnectionManager>>) -> Result<Json<()>> {
    let redis_conn = redis_pool.get()?;
    let cookie_value = get_cookie_value();
    let cookie_value_clone = cookie_value.clone(); // must clone prior to async call
    let account_id = db.run(move |conn| queries::sign_in(sign_in_post.email, sign_in_post.password, conn)).await?;
    store_account_id_in_redis(cookie_value, account_id, redis_conn)?;
    jar.add_private(Cookie::new(COOKIE_KEY, cookie_value_clone));
    Ok(Json(()))
}

#[post("/v1/sign-out")]
pub async fn sign_out(jar: &CookieJar<'_>, redis_pool: &State<Pool<RedisConnectionManager>>) -> Result<Json<()>> {
    let redis_conn = redis_pool.get()?;
    let cookie = match jar.get_private(COOKIE_KEY) {
        Some(cookie) => cookie,
        None => return Ok(Json(())),
    };
    remove_account_id_from_redis(String::from(cookie.value()), redis_conn)?;
    jar.remove_private(cookie);
    Ok(Json(()))
}

#[post("/v1/sign-up", data = "<account_post>")]
pub async fn sign_up(account_post: AccountPost, db: DbConn, jar: &CookieJar<'_>, redis_pool: &State<Pool<RedisConnectionManager>>) -> Result<Json<()>> {
    let redis_conn = redis_pool.get()?;
    let cookie_value = get_cookie_value();
    let cookie_value_clone = cookie_value.clone(); // must clone prior to async call
    let account_id = db.run(move |conn| conn.build_transaction().run(|| queries::sign_up(account_post, conn))).await?;
    store_account_id_in_redis(cookie_value, account_id, redis_conn)?;
    jar.add_private(Cookie::new(COOKIE_KEY, cookie_value_clone));
    Ok(Json(()))
}

fn get_cookie_value() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(COOKIE_LENGTH)
        .map(char::from)
        .collect()
}

fn store_account_id_in_redis(key: String, account_id: i32, mut conn: PooledConnection<RedisConnectionManager>) -> Result<()> {
    redis::cmd("SET")
       .arg(key)
       .arg(account_id)
       .query::<()>(conn.deref_mut())
       .map_err(Error::from)
}

fn remove_account_id_from_redis(key: String, mut conn: PooledConnection<RedisConnectionManager>) -> Result<()> {
    redis::cmd("DEL")
       .arg(key)
       .query::<()>(conn.deref_mut())
       .map_err(Error::from)
}
