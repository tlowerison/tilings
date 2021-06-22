use crate::auth::APIKeyError;
use diesel::result::Error as DieselError;
use r2d2_redis::{r2d2::Error as R2D2Error, redis::RedisError};
use rocket::{
    http::{ContentType, Status},
    serde::json::Json,
};

pub const API_KEY_INVALID_ERR_MSG: &'static str = "API key invalid.";
pub const API_KEY_MISSING_ERR_MSG: &'static str = "API key missing.";
pub const INTERNAL_SERVER_ERR_MSG: &'static str = "Internal server error.";
pub const UNAUTHORIZED_ERR_MSG: &'static str = "Unauthorized.";

#[derive(Debug)]
pub enum Error {
    APIKey(APIKeyError),
    Custom(Status, String),
    Default,
    Diesel(DieselError),
    R2D2(R2D2Error),
    Redis(RedisError),
    Unauthorized,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::APIKey(err) => write!(f, "{}", err),
            Error::Custom(_, msg) => write!(f, "{}", msg),
            Error::Default => write!(f, ""),
            Error::Diesel(err) => write!(f, "{}", err),
            Error::R2D2(err) => write!(f, "{}", err),
            Error::Redis(err) => write!(f, "{}", err),
            Error::Unauthorized => write!(f, "{}", UNAUTHORIZED_ERR_MSG),
        }
    }
}

impl std::error::Error for Error {}

#[macro_export]
macro_rules! error_type {
    ($($name:ident),*) => {
        mashup! { $(
            Error[$name "Error"] = $name Error;
            Result[$name "Result"] = $name Result;
        )* }

        $(
            Error! {
                impl From<$name "Error"> for Error {
                    fn from(err: $name "Error") -> Error {
                        Error::$name(err)
                    }
                }
            }
        )*
    }
}

error_type!{
    APIKey,
    Diesel,
    R2D2,
    Redis
}

pub type Result<T> = std::result::Result<T, Error>;

struct Response {
    pub message: String,
    pub status: Status,
}

impl From<Error> for Response {
    fn from(err: Error) -> Response {
        match err {
            Error::APIKey(err) => match err {
                APIKeyError::Invalid => Response {
                    message: String::from(API_KEY_INVALID_ERR_MSG),
                    status: Status::Unauthorized,
                },
                APIKeyError::Missing => Response {
                    message: String::from(API_KEY_MISSING_ERR_MSG),
                    status: Status::Unauthorized,
                },
            },
            Error::Unauthorized =>  Response {
                message: String::from(UNAUTHORIZED_ERR_MSG),
                status: Status::Unauthorized,
            },
            _ => Response {
                message: String::from(INTERNAL_SERVER_ERR_MSG),
                status: Status::InternalServerError,
            },
        }
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for Error {
    fn respond_to(self, req: &'r rocket::request::Request<'_>) -> rocket::response::Result<'static> {
        let response = Response::from(self);
        rocket::response::Response::build_from(Json(response.message).respond_to(&req).unwrap())
            .status(response.status)
            .header(ContentType::JSON)
            .ok()
    }
}
