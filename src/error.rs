use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use derive_more::{Display, Error};
use log::error;
use mongodb::bson;
use utoipa::ToSchema;
use validator::ValidationErrors;

#[derive(Debug, Display, Error, ToSchema)]
pub enum AppError {
    #[display(fmt = "Empty List")]
    Empty,
    #[display(fmt = "Entity not found")]
    NotFound,
    #[display(fmt = "Failed to parse id (id not valid)")]
    CannotParseObjId,
    #[display(fmt = "ImbdId malformed (imbdId not valid)")]
    WrongImdbId,
    #[display(fmt = "An entity with that id already exists.")]
    AlreadyExists,
    #[display(fmt = "There is no entity with that id.")]
    NotExists,
    #[display(fmt = "The imdbId passed is already in use by another entity.")]
    ImdbIdInUse,
    #[display(fmt = "The field passed not exists in entity or is not allowed.")]
    FieldNotAllowed,
    #[display(fmt = "An internal server error ocurred.")]
    InternalServerError,
    #[display(fmt = "Error in Validation: ({_0})")]
    ValidationAppError(#[error(not(source))] String),
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
            AppError::AlreadyExists => StatusCode::BAD_REQUEST,
            AppError::NotExists => StatusCode::BAD_REQUEST,
            AppError::ImdbIdInUse => StatusCode::BAD_REQUEST,
            AppError::FieldNotAllowed => StatusCode::BAD_REQUEST,
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationAppError(_) => StatusCode::BAD_REQUEST,
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

impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        let msg = format_validation_errors(&err);
        error!("Error in Validation: [{msg}]");
        AppError::ValidationAppError(msg)
    }
}

fn format_validation_errors(errors: &ValidationErrors) -> String {
    format_errors(errors, 0)
}

fn format_errors(errors: &ValidationErrors, depth: usize) -> String {
    let indent = " ".repeat(depth);
    errors
        .errors()
        .iter()
        .map(|(field, error)| match error {
            validator::ValidationErrorsKind::Struct(nested_errors) => {
                format!(
                    "{}{}: {}",
                    indent,
                    field,
                    format_errors(nested_errors, depth + 1)
                )
            }
            validator::ValidationErrorsKind::List(list_errors) => {
                let nested = list_errors
                    .iter()
                    .map(|(index, errors)| {
                        format!(
                            "{} [{}]: {}",
                            indent,
                            index,
                            format_errors(errors, depth + 2)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(";");
                format!("{}{}: {}", indent, field, nested)
            }
            validator::ValidationErrorsKind::Field(field_errors) => {
                let messages = field_errors
                    .iter()
                    .map(|error| {
                        error
                            .message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "Unknown error".to_string())
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}{}: {}", indent, field, messages)
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}
