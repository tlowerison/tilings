#[macro_use] extern crate rocket;

use api::BASE_PATH;
use lazy_static::lazy_static;
use r2d2_redis::{r2d2::Pool, redis, RedisConnectionManager};
use regex::Regex;
use result::{Error, Result};
use rocket::{
    Data,
    fairing::{Fairing, Info, Kind},
    http::uri::Origin,
    request::{Outcome, Request},
    serde::json::Json,
    State,
};
use std::{
    ops::DerefMut,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const KEY_PREFIX: &'static str = "RATE_LIMITER";
lazy_static!{
    static ref DEFAULT_MAX_COUNT_IN_WINDOW: usize = std::env::var("RATE_LIMIT_DEFAULT_MAX_COUNT_IN_WINDOW").unwrap().parse().unwrap();
    static ref SEARCH_MAX_COUNT_IN_WINDOW: usize = std::env::var("RATE_LIMIT_SEARCH_MAX_COUNT_IN_WINDOW").unwrap().parse().unwrap();
    static ref DURATION: Duration = Duration::from_secs(30);
    static ref FAILURE_PATH: String = format!("{}{}", BASE_PATH, "/v1/fail-rait-limit");
    static ref FAILURE_URI: Origin<'static> = Origin::parse(&*FAILURE_PATH).unwrap();
    static ref SEARCH_PATHS_RE: Regex = Regex::new(&format!(
        "^({})",
        vec![
            "/v1/omni-search",
            "/v1/tiling-search",
        ]
            .into_iter()
            .map(|path| format!("{}{}", BASE_PATH, path))
            .collect::<Vec<String>>()
            .join("|"),
    )).unwrap();
}

pub struct RateLimiter {}

#[rocket::async_trait]
impl Fairing for RateLimiter {
    fn info(&self) -> Info {
        Info {
            name: "Rate Limiter",
            kind: Kind::Request
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        let path = req.uri().path();
        let ip = match req.client_ip() {
            Some(ip) => ip,
            _ => {
                req.set_uri((*FAILURE_URI).clone());
                return
            },
        };

        let redis_pool = match req.guard::<&State<Pool<RedisConnectionManager>>>().await {
            Outcome::Success(redis_pool) => redis_pool,
            _ => {
                req.set_uri((*FAILURE_URI).clone());
                return
            },
        };
        let mut redis_conn = match redis_pool.get() {
            Ok(redis_conn) => redis_conn,
            _ => {
                req.set_uri((*FAILURE_URI).clone());
                return
            },
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let duration = *DURATION;
        let window = (now.as_secs() / duration.as_secs()) * duration.as_secs();

        let is_search = SEARCH_PATHS_RE.is_match(path.as_str());

        let key = if is_search {
            format!("{}:{}:search:{}", KEY_PREFIX, ip, window)
        } else {
            format!("{}:{}:{}", KEY_PREFIX, ip, window)
        };

        let (count,): (usize,) = match redis::pipe()
            .atomic()
            .incr(&key, 1_usize)
            .expire(&key, duration.as_secs() as usize)
            .ignore()
            .query(redis_conn.deref_mut()) {
                Ok(val) => val,
                _ => {
                    req.set_uri((*FAILURE_URI).clone());
                    return
                },
            };

        if count >= (if is_search { *SEARCH_MAX_COUNT_IN_WINDOW } else { *DEFAULT_MAX_COUNT_IN_WINDOW }) {
            req.set_uri((*FAILURE_URI).clone());
            return
        }
    }
}

#[get("/v1/fail-rait-limit")]
pub async fn fail_rait_limit() -> Result<Json<()>> {
    Err(Error::RateLimit)
}
