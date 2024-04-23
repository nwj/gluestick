use crate::views::{InternalServerErrorTemplate, NotFoundTemplate};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub mod pastes;

pub async fn not_found() -> Result<(), AppError> {
    Err(AppError::NotFound)
}

pub async fn internal_server_error() -> Result<(), AppError> {
    Err(AppError::Generic)
}

// This represents any error that we could get out of a route handler
// and maps them to an appropriate status code and response template.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("generic error")]
    Generic,

    #[error(transparent)]
    Database(#[from] crate::db::DatabaseError),

    #[error("resource not found")]
    NotFound,
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Generic | AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn template(&self) -> ErrorTemplate {
        match self {
            AppError::Generic | AppError::Database(_) => {
                ErrorTemplate::InternalServerError(InternalServerErrorTemplate {})
            }
            AppError::NotFound => ErrorTemplate::NotFound(NotFoundTemplate {}),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status_code(), self.template()).into_response()
    }
}

enum ErrorTemplate {
    NotFound(NotFoundTemplate),
    InternalServerError(InternalServerErrorTemplate),
}

impl IntoResponse for ErrorTemplate {
    fn into_response(self) -> Response {
        match self {
            ErrorTemplate::InternalServerError(template) => template.into_response(),
            ErrorTemplate::NotFound(template) => template.into_response(),
        }
    }
}
