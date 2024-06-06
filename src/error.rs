use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use derive_more::{Display, Error};
use log::error;
use mongodb::bson;

#[derive(Debug, Display, Error)]
pub enum AppError {
    #[display(fmt = "Empty List")]
    Empty,
    #[display(fmt = "Entity not found")]
    NotFound,
    #[display(fmt = "Failed to parse id (id not valid)")]
    CannotParseObjId,
    #[display(fmt = "ImbdId malformed (imbdId not valid)")]
    WrongImdbId,
    #[display(fmt = "An internal server error ocurred.")]
    InternalServerError,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            AppError::Empty => HttpResponse::build(self.status_code()).finish(),
            _ => HttpResponse::build(self.status_code()).json(self.to_string()),
        }
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::Empty => StatusCode::NO_CONTENT,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::CannotParseObjId => StatusCode::BAD_REQUEST,
            AppError::WrongImdbId => StatusCode::BAD_REQUEST,
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<bson::oid::Error> for AppError {
    fn from(value: bson::oid::Error) -> Self {
        let _ = value;
        error!("Error parsing ObjectId from string");
        AppError::CannotParseObjId
    }
}
