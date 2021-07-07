use percent_encoding;
use result::{Error, Result};
use rocket::http::Status;

const INVALID_QUERY_STRING_ERR_MSG: &'static str = "Invalid query string.";

pub fn clamp_optional(max: u32, value: Option<u32>) -> u32 {
    match value {
        Some(value) => if value >= max { max } else { value },
        None => max,
    }
}

pub fn percent_decode(query: String) -> Result<String> {
    percent_encoding::percent_decode(query.as_bytes())
        .decode_utf8()
        .map(String::from)
        .or(Err(Error::Custom(Status::BadRequest, String::from(INVALID_QUERY_STRING_ERR_MSG))))
}
