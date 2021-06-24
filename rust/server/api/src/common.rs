use auth::Role;
use lazy_static::lazy_static;
use percent_encoding;
use result::{Error, Result};
use rocket::http::Status;
use std::collections::hash_set::HashSet;

const INVALID_QUERY_STRING_ERR_MSG: &'static str = "Invalid query string.";

lazy_static! {
    pub static ref ALLOWED_EDITOR_ROLES: HashSet<Role> = [Role::Editor, Role::Admin].iter().cloned().collect();
    pub static ref ALLOWED_ADMIN_ROLES: HashSet<Role> = [Role::Admin].iter().cloned().collect();
}

pub fn clamp_optional(max: u32, value: Option<u32>) -> u32 {
    match value {
        Some(value) => if value >= max { max } else { value },
        None => max,
    }
}

pub fn percent_decode(query: String) -> Result<String> {
    println!("{}", query);
    percent_encoding::percent_decode(query.as_bytes())
        .decode_utf8()
        .map(|e| { println!("{}", e); String::from(e) })
        .or(Err(Error::Custom(Status::BadRequest, String::from(INVALID_QUERY_STRING_ERR_MSG))))
}
