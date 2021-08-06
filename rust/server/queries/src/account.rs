use auth::{APIKeyClaims, AuthAccount};
use argon2::{self, Config};
use chrono::{Duration, offset::Utc};
use diesel::{PgConnection, prelude::*};
use lazy_static::lazy_static;
use models::*;
use rand::{distributions::Alphanumeric, Rng};
use result::{Error, Result};
use rocket::http::Status;
use schema::*;
use validator::validate_email;

pub const MAX_EMAIL_LENGTH: usize = 100;
const MAX_EMAIL_ERR_MSG: &'static str = "Email must be 100 characters or less.";
const INVALID_EMAIL_ERR_MSG: &'static str = "Invalid email.";

pub const MIN_PASSWORD_LENGTH: usize = 10;
pub const MAX_PASSWORD_LENGTH: usize = 100;
const MIN_PASSWORD_ERR_MSG: &'static str = "Password must be 10 characters or more.";
const MAX_PASSWORD_ERR_MSG: &'static str = "Password must be 100 characters or less.";
const INVALID_PASSWORD_ERR_MSG: &'static str = "Invalid password.";

pub const MIN_DISPLAY_NAME_LENGTH: usize = 3;
const MIN_DISPLAY_NAME_ERR_MSG: &'static str = "Display Name must be 3 characters or more.";

pub const MAX_DISPLAY_NAME_LENGTH: usize = 100;
const MAX_DISPLAY_NAME_ERR_MSG: &'static str = "Display Name must be 100 characters or less.";

const EMAIL_DISPLAY_NAME_TAKEN_ERR_MSG: &'static str = "Email / Display Name taken.";

const VERIFICATION_CODE_LENGTH: usize = 32;

pub const VALID_PASSWORD_RESET_CODE_DURATION_IN_MINUTES: i64 = 15;

lazy_static!{
    static ref VALID_PASSWORD_RESET_CODE_DURATION: Duration = Duration::minutes(VALID_PASSWORD_RESET_CODE_DURATION_IN_MINUTES);
}

fn get_code() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(VERIFICATION_CODE_LENGTH)
        .map(char::from)
        .collect()
}

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

pub fn get_account_by_email(email: String, conn: &PgConnection) -> Result<Account> {
    account::table.filter(account::email.eq(email))
        .get_result(conn)
        .map_err(Error::from)
}

pub fn get_account_by_password_reset_code(password_reset_code: String, conn: &PgConnection) -> Result<Account> {
    account::table
        .filter(account::password_reset_code.eq(password_reset_code))
        .filter(account::password_reset_code_timestamp.gt(Utc::now().naive_utc() - *VALID_PASSWORD_RESET_CODE_DURATION))
        .get_result(conn)
        .map_err(Error::from)
}

pub fn get_account_tilings(auth_account: AuthAccount, conn: &PgConnection) -> Result<Vec<FullTiling>> {
    let tiling_ids = tiling::table
        .filter(tiling::owner_id.eq(auth_account.id))
        .select(tiling::id)
        .get_results(conn)
        .map_err(Error::from)?;

    FullTiling::find_batch(tiling_ids, conn)
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

pub fn reset_password(account_id: i32, password: String, conn: &PgConnection) -> Result<()> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MIN_PASSWORD_ERR_MSG),
        ));
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MAX_PASSWORD_ERR_MSG),
        ));
    }

    let salt: [u8; 32] = rand::thread_rng().gen();
    let config = Config::default();
    let password_hash = argon2::hash_encoded(password.as_bytes(), &salt, &config).unwrap();

    AccountPatch {
        id: account_id,
        verified: None,
        email: None,
        password: Some(password_hash),
        display_name: None,
        verification_code: None,
        password_reset_code: None,
        password_reset_code_timestamp: None,
    }.update(conn)?;
    Ok(())
}

pub fn reset_password_reset_code(account_id: i32, conn: &PgConnection) -> Result<Account> {
    AccountPatch {
        id: account_id,
        password_reset_code: Some(Some(get_code())),
        password_reset_code_timestamp: Some(Some(Utc::now().naive_utc())),
        verified: None,
        email: None,
        password: None,
        display_name: None,
        verification_code: None,
    }.update(conn)
}

pub fn reset_verification_code(account_id: i32, conn: &PgConnection) -> Result<Account> {
    AccountPatch {
        id: account_id,
        verification_code: Some(Some(get_code())),
        verified: None,
        email: None,
        password: None,
        display_name: None,
        password_reset_code: None,
        password_reset_code_timestamp: None,
    }.update(conn)
}

pub fn sign_up(sign_up_post: SignUpPost, conn: &PgConnection) -> Result<Account> {
    if sign_up_post.email.len() > MAX_EMAIL_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MAX_EMAIL_ERR_MSG),
        ));
    }

    if sign_up_post.password.len() < MIN_PASSWORD_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MIN_PASSWORD_ERR_MSG),
        ));
    }

    if sign_up_post.password.len() > MAX_PASSWORD_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MAX_PASSWORD_ERR_MSG),
        ));
    }

    if sign_up_post.display_name.len() < MIN_DISPLAY_NAME_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MIN_DISPLAY_NAME_ERR_MSG),
        ));
    }

    if sign_up_post.display_name.len() > MAX_DISPLAY_NAME_LENGTH {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(MAX_DISPLAY_NAME_ERR_MSG),
        ));
    }

    if !validate_email(&sign_up_post.email) {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(INVALID_EMAIL_ERR_MSG),
        ));
    }

    let salt: [u8; 32] = rand::thread_rng().gen();
    let config = Config::default();
    let password_hash = argon2::hash_encoded(sign_up_post.password.as_bytes(), &salt, &config).unwrap();

    let account = AccountPost {
        email: sign_up_post.email,
        password: password_hash,
        display_name: sign_up_post.display_name,
        verified: false,
        verification_code: Some(get_code()),
        password_reset_code: None,
        password_reset_code_timestamp: None,
    }.insert(conn)
        .or(Err(Error::Custom(
            Status::BadRequest,
            String::from(EMAIL_DISPLAY_NAME_TAKEN_ERR_MSG),
        )))?;

    AccountRolePost {
        account_id: account.id,
        role_id: RoleEnum::Editor.as_role_id(),
    }.insert(conn)?;

    Ok(account)
}

pub fn sign_in(email: String, password: String, conn: &PgConnection) -> Result<Account> {
    let account: Account = account::table.filter(account::email.eq(email))
        .get_result(conn)
        .or(Err(Error::Custom(Status::BadRequest, String::from(INVALID_EMAIL_ERR_MSG))))?;

    let is_password_correct = argon2::verify_encoded(&account.password, password.as_bytes())
        .or(Err(Error::Custom(Status::BadRequest, String::from(INVALID_PASSWORD_ERR_MSG))))?;

    if is_password_correct {
        return Ok(account)
    } else {
        return Err(Error::Custom(
            Status::BadRequest,
            String::from(INVALID_PASSWORD_ERR_MSG),
        ))
    }
}

pub fn verify(verification_code: String, conn: &PgConnection) -> Result<bool> {
    let existing_account: Account = account::table.filter(account::verification_code.eq(verification_code))
        .get_result(conn)?;

    if existing_account.verified {
        return Ok(true)
    }

    AccountPatch {
        id: existing_account.id,
        verified: Some(true),
        verification_code: Some(None),
        email: None,
        password: None,
        display_name: None,
        password_reset_code: None,
        password_reset_code_timestamp: None,
    }.update(conn)?;

    Ok(true)
}
