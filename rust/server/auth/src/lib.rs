pub const COOKIE_KEY: &'static str = "tilings_account_id";

#[cfg(not(target_arch = "wasm32"))]
mod auth {
    use super::*;
    use argon2;
    use base64;
    use db_conn::DbConn;
    use diesel::{self, PgConnection, prelude::*};
    use lazy_static::lazy_static;
    use models::*;
    use r2d2_redis::{r2d2::Pool, redis, RedisConnectionManager};
    use result::{APIKeyError, Error, Result};
    use rocket::{
        http::Status,
        request::{Outcome, Request, FromRequest},
        State,
    };
    use schema::accountrole;
    use serde::{Deserialize, Serialize};
    use std::{
        collections::hash_set::HashSet,
        ops::DerefMut,
    };

    pub const AUTHORIZATION_HEADER_KEY: &'static str = "Authorization";
    pub const SECRET: &'static str = "JWT_TOKEN";
    pub const TOKEN_DURATION_IN_SECONDS: i64 = 10 * 365 * 24 * 60 * 60;
    lazy_static! {
        static ref AUTHORIZATION_HEADER_VALUE_PREFIX_END_INDEX: usize = "Bearer ".len();
    }

    pub struct AuthAccount {
        pub id: i32,
        account: Option<Account>,
        roles: Option<HashSet<RoleEnum>>,
    }

    impl<'a> AuthAccount {
        pub fn new(id: i32) -> AuthAccount {
            AuthAccount { id, account: None, roles: None }
        }

        pub fn allowed(&mut self, allowed_roles: &HashSet<RoleEnum>, conn: &PgConnection) -> Result<bool> {
            self.pull_roles(conn)?;
            if AuthAccount::has_intersection(self.roles.as_ref().unwrap(), allowed_roles) && self.verified(conn)? {
                Ok(true)
            } else {
                Err(Error::Unauthorized)
            }
        }

        pub fn can_edit(&mut self, owned: Owned, id: i32, conn: &PgConnection) -> Result<bool> {
            if let Ok(_) = self.allowed(&ALLOWED_ADMIN_ROLES, conn) {
                return Ok(true)
            }

            self.allowed(&ALLOWED_EDITOR_ROLES, conn).or(Err(Error::Unauthorized))?;

            let owner_id = owned.get_owner_id(id, conn)?;
            if let Some(owner_id) = owner_id {
                if owner_id == self.id {
                    return Ok(true)
                }
            }

            Err(Error::Unauthorized)
        }

        pub fn get_account(&'a mut self, conn: &PgConnection) -> Result<&'a Account> {
            if let None = self.account {
                self.account = Some(Account::find(self.id, conn)?);
            }
            Ok(self.account.as_ref().unwrap())
        }

        fn has_intersection(roles: &HashSet<RoleEnum>, allowed_roles: &HashSet<RoleEnum>) -> bool {
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
                .filter_map(AccountRole::as_role_enum)
                .collect()
            );

            Ok(())
        }

        fn verified(&mut self, conn: &PgConnection) -> Result<bool> {
            self.get_account(conn)?;
            if self.account.as_ref().unwrap().verified {
                Ok(true)
            } else {
                Err(Error::Unauthorized)
            }
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
}

#[cfg(not(target_arch = "wasm32"))]
pub use self::auth::*;
