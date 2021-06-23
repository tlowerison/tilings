use crate::{
    connection::DbConn,
    models::*,
    result::{Error, Result},
    schema::accountrole,
};
use argon2;
use base64;
use diesel::{self, PgConnection, prelude::*};
use lazy_static::lazy_static;
use r2d2_redis::{r2d2::Pool, RedisConnectionManager};
use rocket::{
    http::Status,
    request::{Outcome, Request, FromRequest},
    State,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_set::HashSet,
    hash::Hash,
    ops::DerefMut,
};

pub const AUTHORIZATION_HEADER_KEY: &'static str = "Authorization";
pub const COOKIE_KEY: &'static str = "tilings_account_id";
pub const SECRET: &'static str = "JWT_TOKEN";
pub const TOKEN_DURATION_IN_SECONDS: i64 = 10 * 365 * 24 * 60 * 60;
lazy_static! {
    static ref AUTHORIZATION_HEADER_VALUE_PREFIX_END_INDEX: usize = "Bearer ".len();
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Role {
    Admin,
    Editor,
    ReadOnly,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "Admin"),
            Role::Editor => write!(f, "Editor"),
            Role::ReadOnly => write!(f, "ReadOnly"),
        }
    }
}

fn as_role(account_role: AccountRole) -> Option<Role> {
    match account_role.role_id {
        1 => Some(Role::ReadOnly),
        2 => Some(Role::Editor),
        3 => Some(Role::Admin),
        _ => None,
    }
}

pub struct AuthAccount {
    pub id: i32,
    account: Option<Account>,
    roles: Option<HashSet<Role>>,
}

impl<'a> AuthAccount {
    pub fn new(id: i32) -> AuthAccount {
        AuthAccount { id, account: None, roles: None }
    }

    fn has_intersection(roles: &HashSet<Role>, allowed_roles: &HashSet<Role>) -> bool {
        for role in roles.iter() {
            if allowed_roles.contains(role) {
                return true
            }
        }
        return false
    }

    fn pull_roles(&mut self, conn: &PgConnection) -> Result<()> {
        if let Some(_) = &self.roles {
            return Ok(())
        }
        let account_roles = accountrole::table.filter(accountrole::account_id.eq(&self.id))
            .load(conn)?;

        self.roles = Some(account_roles
            .into_iter()
            .filter_map(|account_role| as_role(account_role))
            .collect()
        );

        Ok(())
    }

    pub fn get_account(&'a mut self, conn: &PgConnection) -> Result<&'a Account> {
        if let None = self.account {
            self.account = Some(Account::find(self.id, conn)?);
        }
        Ok(self.account.as_ref().unwrap())
    }

    pub fn allowed(&mut self, allowed_roles: &HashSet<Role>, conn: &PgConnection) -> Result<bool> {
        self.pull_roles(conn)?;
        if AuthAccount::has_intersection(self.roles.as_ref().unwrap(), allowed_roles) {
            Ok(true)
        } else {
            Err(Error::Unauthorized)
        }
    }

    pub fn verified(&mut self, conn: &PgConnection) -> Result<bool> {
        self.get_account(conn)?;
        if self.account.as_ref().unwrap().verified {
            Ok(true)
        } else {
            Err(Error::Unauthorized)
        }
    }
}

#[derive(Debug)]
pub enum APIKeyError {
    Invalid,
    Missing,
}

impl std::fmt::Display for APIKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            APIKeyError::Invalid => write!(f, "Invalid API key."),
            APIKeyError::Missing => write!(f, "Missing API Key."),
        }
    }
}

impl APIKeyError {
    pub fn outcome(self) -> Outcome<AuthAccount, Error> {
        Outcome::Failure((Status::Unauthorized, Error::APIKey(self)))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct APIKeyClaims {
    pub email: String,
    pub api_key: String,
}

impl APIKeyClaims {
    fn decode(encoded: &str) -> Result<APIKeyClaims> {
        let decoded = base64::decode(encoded).or(Err(Error::APIKey(APIKeyError::Invalid)))?;
        let decoded = std::str::from_utf8(decoded.as_slice()).or(Err(Error::APIKey(APIKeyError::Invalid)))?;
        serde_json::from_str::<APIKeyClaims>(decoded)
            .or(Err(Error::APIKey(APIKeyError::Invalid)))
    }

    pub fn encode(self) -> Result<String> {
        let serialized = serde_json::to_string(&self).or(Err(Error::Default))?;
        Ok(base64::encode(String::from(serialized)))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthAccount {
    type Error = Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one(AUTHORIZATION_HEADER_KEY) {
            None => match req.cookies().get_private(COOKIE_KEY) {
                Some(cookie) => {
                    let redis_pool = match req.guard::<&State<Pool<RedisConnectionManager>>>().await {
                        Outcome::Success(redis_pool) => redis_pool,
                        _ => return Outcome::Failure((Status::InternalServerError, Error::Default)),
                    };
                    let mut redis_conn = match redis_pool.get() {
                        Ok(redis_conn) => redis_conn,
                        Err(err) => return Outcome::Failure((Status::InternalServerError, Error::from(err))),
                    };
                    let cookie_key = cookie.value();
                    let cookie_value = match redis::cmd("GET")
                       .arg(cookie_key)
                       .query::<String>(redis_conn.deref_mut())
                    {
                        Ok(cookie_value) => cookie_value,
                        Err(err) => return Outcome::Failure((Status::InternalServerError, Error::from(err))),
                    };
                    let account_id = cookie_value.parse::<i32>();
                    return match account_id {
                        Ok(account_id) => Outcome::Success(AuthAccount::new(account_id)),
                        _ => APIKeyError::Invalid.outcome(),
                    }
                },
                None => APIKeyError::Missing.outcome(),
            },
            Some(key) => {
                let token = key
                    .chars()
                    .into_iter()
                    .skip(*AUTHORIZATION_HEADER_VALUE_PREFIX_END_INDEX)
                    .collect::<String>();

                let api_key_claims = match APIKeyClaims::decode(&token) {
                    Ok(api_key_claims) => api_key_claims,
                    Err(_) => return APIKeyError::Invalid.outcome(),
                };

                let email = api_key_claims.email;
                let api_key_content = api_key_claims.api_key;

                let db = match req.guard::<DbConn>().await {
                    Outcome::Success(db) => db,
                    _ => return Outcome::Failure((Status::InternalServerError, Error::Default)),
                };

                let api_key = match db.run(move |conn| APIKey::find_by_email(email, conn)).await {
                    Ok(api_key) => api_key,
                    _ => return APIKeyError::Invalid.outcome(),
                };

                let is_match = match argon2::verify_encoded(&api_key.content, api_key_content.as_bytes()) {
                    Ok(is_match) => is_match,
                    _ => return Outcome::Failure((Status::InternalServerError, Error::Default)),
                };

                if is_match {
                    Outcome::Success(AuthAccount::new(api_key.account_id))
                } else {
                    APIKeyError::Invalid.outcome()
                }
            },
        }
    }
}
