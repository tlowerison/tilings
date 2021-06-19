use crate::result::*;
use rocket::{
    http::{ContentType, Status},
    serde::json::Json,
};
use serde::Serialize;

pub const INTERNAL_SERVER_ERR_MSG: &'static str = "Internal server error.";

pub struct Response<T: Serialize> {
    pub data: Option<T>,
    pub message: Option<String>,
    pub status: Status,
}

impl<T: Serialize> From<DbResult<T>> for Response<T> {
    fn from(db_result: DbResult<T>) -> Response<T> {
        Response::from(match db_result {
            Ok(data) => Ok(data),
            Err(diesel_error) => Err(Error::Diesel(diesel_error)),
        })
    }
}

impl<T: Serialize> From<Result<T>> for Response<T> {
    fn from(result: Result<T>) -> Response<T> {
        match result {
            Ok(data) => Response {
                data: Some(data),
                message: None,
                status: Status::Ok,
            },
            Err(err) => match err {
                Error::Custom(status, message) => Response {
                    data: None,
                    message: Some(message),
                    status,
                },
                Error::Diesel(_diesel_error) => Response {
                    data: None,
                    message: Some(String::from(INTERNAL_SERVER_ERR_MSG)),
                    status: Status::InternalServerError,
                },
            },
        }
    }
}

impl<'r, T: Serialize> rocket::response::Responder<'r, 'static> for Response<T> {
    fn respond_to(self, req: &'r rocket::request::Request<'_>) -> rocket::response::Result<'static> {
        let base_response = match self.data {
            Some(data) => Json(data).respond_to(&req).unwrap(),
            None => match self.message {
                Some(message) => Json(message).respond_to(&req).unwrap(),
                None => rocket::response::Response::new(),
            },
        };
        rocket::response::Response::build_from(base_response)
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}
