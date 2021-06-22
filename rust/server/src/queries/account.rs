use crate::{
    auth::{APIKeyClaims, AuthAccount},
    models::*,
    result::{Error, Result},
    schema::{account, apikey},
};
use argon2::{self, Config};
use diesel::{PgConnection, prelude::*};
use rand::{distributions::Alphanumeric, Rng};
use rocket::http::Status;
use validator::validate_email;

pub const MAX_EMAIL_LENGTH: usize = 100;
const MAX_EMAIL_ERR_MSG: &'static str = "Email must be 100 characters or less.";
const INVALID_EMAIL_ERR_MSG: &'static str = "Invalid email.";

pub const MIN_PASSWORD_LENGTH: usize = 10;
pub const MAX_PASSWORD_LENGTH: usize = 100;
const MIN_PASSWORD_ERR_MSG: &'static str = "Password must be 10 characters or more.";
const MAX_PASSWORD_ERR_MSG: &'static str = "Password must be 100 characters or less.";
const INVALID_PASSWORD_ERR_MSG: &'static str = "Invalid password.";

pub const MAX_DISPLAY_NAME_LENGTH: usize = 100;
const MAX_DISPLAY_NAME_ERR_MSG: &'static str = "Display Name must be 100 characters or less.";

const EMAIL_DISPLAY_NAME_TAKEN_ERR_MSG: &'static str = "Email / Display Name taken.";

// true if email is available
pub fn check_email(email: String, conn: &PgConnection) -> Result<bool> {
    let count: Option<i64> = account::table.filter(account::email.eq(email))
        .count()
        .get_result(conn)
        .optional()?;
    Ok(match count { Some(count) => count == 0, None => false })
}

// true if display_name is available
pub fn check_display_name(display_name: String, conn: &PgConnection) -> Result<bool> {
    let count: Option<i64> = account::table.filter(account::display_name.eq(display_name))
        .count()
        .get_result(conn)
        .optional()?;
    Ok(match count { Some(count) => count == 0, None => false })
}

pub fn reset_api_key(mut auth_account: AuthAccount, conn: &PgConnection) -> Result<String> {
    diesel::delete(apikey::table.filter(apikey::account_id.eq(auth_account.id)))
        .execute(conn)?;

    let email = auth_account.get_account(conn)?.email.clone();

    let api_key: String = rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(100)
        .map(char::from)
        .collect();

    let salt: [u8; 32] = rand::thread_rng().gen();
    let config = Config::default();
    let api_key_hash = argon2::hash_encoded(&api_key.as_bytes(), &salt, &config).unwrap();

    APIKeyPost { account_id: auth_account.id, content: api_key_hash }
        .insert(conn)?;

    APIKeyClaims { email, api_key }.encode()
}

pub fn sign_up(account_post: AccountPost, conn: &PgConnection) -> Result<i32> {
    if account_post.email.len() > MAX_EMAIL_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MAX_EMAIL_ERR_MSG),
        ));
    }

    if account_post.password.len() < MIN_PASSWORD_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MIN_PASSWORD_ERR_MSG),
        ));
    }

    if account_post.password.len() > MAX_PASSWORD_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MAX_PASSWORD_ERR_MSG),
        ));
    }

    if account_post.display_name.len() > MAX_DISPLAY_NAME_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MAX_DISPLAY_NAME_ERR_MSG),
        ));
    }

    if !validate_email(&account_post.email) {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(INVALID_EMAIL_ERR_MSG),
        ));
    }

    let salt: [u8; 32] = rand::thread_rng().gen();
    let config = Config::default();
    let password_hash = argon2::hash_encoded(account_post.password.as_bytes(), &salt, &config).unwrap();

    let account = AccountPost {
        email: account_post.email,
        password: password_hash,
        display_name: account_post.display_name,
        verified: false,
    }.insert(conn)
        .or(Err(Error::Custom(
            Status::BadRequest,
            String::from(EMAIL_DISPLAY_NAME_TAKEN_ERR_MSG),
        )))?;

    Ok(account.id)
}

pub fn sign_in(email: String, password: String, conn: &PgConnection) -> Result<i32> {
    let account: Account = account::table.filter(account::email.eq(email))
        .get_result(conn)
        .or(Err(Error::Custom(Status::BadRequest, String::from(INVALID_EMAIL_ERR_MSG))))?;

    let is_password_correct = argon2::verify_encoded(&account.password, password.as_bytes())
        .or(Err(Error::Custom(Status::BadRequest, String::from(INVALID_PASSWORD_ERR_MSG))))?;

    if is_password_correct {
        return Ok(account.id)
    } else {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(INVALID_PASSWORD_ERR_MSG),
        ))
    }
}

pub fn sign_out(_conn: &PgConnection) -> Result<()> {
    Ok(())
}
