use crate::{
    models::*,
    result::Error,
    schema::accountrole,
};
use chrono::offset::Utc;
use diesel::{self, PgConnection, prelude::*};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation};
use lazy_static::lazy_static;
use regex::Regex;
use r2d2_redis::{r2d2::Pool, RedisConnectionManager};
use rocket::{
    http::Status,
    request::{Outcome, Request, FromRequest},
    State,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_set::HashSet,
    hash::{Hash, Hasher},
    ops::DerefMut,
};

pub const SECRET: &'static str = "JWT_TOKEN";
pub const TOKEN_DURATION_IN_SECONDS: i64 = 10 * 365 * 24 * 60 * 60;
pub const AUTHORIZATION_HEADER_KEY: &'static str = "Authorization";
pub const COOKIE_KEY: &'static str = "tilings_account_id";

#[derive(Debug, Clone)]
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

impl PartialEq for Role {
    fn eq(&self, other: &Role) -> bool { self == other }
}

impl Eq for Role {}

impl Hash for Role {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Role::ReadOnly => 1_u8.hash(state),
            Role::Editor => 2_u8.hash(state),
            Role::Admin => 3_u8.hash(state),
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

impl AuthAccount {
    pub fn new(id: i32) -> AuthAccount {
        AuthAccount { id, account: None, roles: None }
    }

    fn has_intersection(roles: &HashSet<Role>, allowed_roles: &HashSet<Role>) -> bool {
        roles.intersection(&allowed_roles).collect::<HashSet<_>>().len() > 0
    }

    fn pull_account(&mut self, conn: &PgConnection) -> Result<(), Error> {
        if let Some(_) = &self.account {
            return Ok(())
        }
        self.account = Some(Account::find(self.id, conn)?);
        Ok(())
    }

    fn pull_roles(&mut self, conn: &PgConnection) -> Result<(), Error> {
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

    pub fn allowed(&mut self, allowed_roles: &HashSet<Role>, conn: &PgConnection) -> Result<bool, Error> {
        self.pull_roles(conn)?;
        if AuthAccount::has_intersection(self.roles.as_ref().unwrap(), allowed_roles) {
            Ok(true)
        } else {
            Err(Error::Unauthorized)
        }
    }

    pub fn verified(&mut self, conn: &PgConnection) -> Result<bool, Error> {
        self.pull_account(conn)?;
        if self.account.as_ref().unwrap().verified {
            Ok(true)
        } else {
            Err(Error::Unauthorized)
        }
    }
}

#[derive(Debug)]
pub enum ApiKeyError {
    Invalid,
    Missing,
}

impl std::fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiKeyError::Invalid => write!(f, "Invalid API key."),
            ApiKeyError::Missing => write!(f, "Missing API Key."),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub exp: i64, // Expiration time (as UTC timestamp). (validate_exp defaults to true in validation)
    pub iat: i64, // Issued at (as UTC timestamp)
    pub sub: i32, // Subject (whom token refers to: Account.id)
}

impl Claims {
    pub fn new(account_id: i32) -> Claims {
        Claims {
            exp: Utc::now().timestamp() + TOKEN_DURATION_IN_SECONDS,
            iat: Utc::now().timestamp(),
            sub: account_id,
        }
    }
}

pub fn encode(claims: Claims) -> jsonwebtoken::errors::Result<String> {
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_ref()),
    )
}

pub fn decode(token: &str) -> jsonwebtoken::errors::Result<TokenData<Claims>> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET.as_ref()),
        &Validation::default(),
    )
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
                        _ => Outcome::Failure((Status::Unauthorized, Error::ApiKey(ApiKeyError::Invalid))),
                    }
                },
                None => Outcome::Failure((Status::Unauthorized, Error::ApiKey(ApiKeyError::Missing))),
            },
            Some(key) => {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"(?<=Bearer: ).*").unwrap();
                }

                let token = match RE.find(key) {
                    Some(matched) => matched.as_str(),
                    None => return Outcome::Failure((Status::Unauthorized, Error::ApiKey(ApiKeyError::Invalid))),
                };

                let token_data = match decode(token) {
                    Ok(token_data) => token_data,
                    Err(_) => return Outcome::Failure((Status::Unauthorized, Error::ApiKey(ApiKeyError::Invalid))),
                };

                if token_data.claims.exp >= Utc::now().timestamp() {
                    Outcome::Success(AuthAccount::new(token_data.claims.sub))
                } else {
                    Outcome::Failure((Status::Unauthorized, Error::ApiKey(ApiKeyError::Invalid)))
                }
            },
        }
    }
}
