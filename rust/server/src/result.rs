use diesel::result::Error as DieselError;
use rocket::http::Status;

#[derive(Debug)]
pub enum Error {
    Custom(Status, String),
    Diesel(DieselError),
}

impl From<DieselError> for Error {
    fn from(diesel_error: DieselError) -> Error {
        Error::Diesel(diesel_error)
    }
}

pub type DbResult<T> = std::result::Result<T, DieselError>;

pub type Result<T> = std::result::Result<T, Error>;
