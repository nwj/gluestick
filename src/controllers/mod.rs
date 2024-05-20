use crate::{
    models::session::Session,
    views::{IndexTemplate, InternalServerErrorTemplate, NotFoundTemplate},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub mod api;
pub mod api_sessions;
pub mod health;
pub mod pastes;
pub mod sessions;
pub mod users;

pub async fn index(session: Option<Session>) -> Result<impl IntoResponse, self::Error> {
    Ok(IndexTemplate { session })
}

pub async fn not_found() -> Result<(), self::Error> {
    Err(self::Error::NotFound)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("malformed request")]
    BadRequest(#[from] validator::ValidationErrors),

    #[error("invalid authentication credentials")]
    Unauthorized,

    #[error("insufficient privileges")]
    Forbidden,

    #[error("resource not found")]
    NotFound,

    #[allow(clippy::enum_variant_names)]
    #[error("internal server error: {0}")]
    InternalServerError(#[from] Box<dyn std::error::Error>),
}

enum ErrorTemplate {
    Blank,
    NotFound(NotFoundTemplate),
    InternalServerError(InternalServerErrorTemplate),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, template) = match self {
            Error::BadRequest(err) => {
                tracing::error!(%err, "bad request");
                (StatusCode::BAD_REQUEST, ErrorTemplate::Blank)
            }

            Error::Unauthorized => (StatusCode::UNAUTHORIZED, ErrorTemplate::Blank),

            Error::Forbidden => (StatusCode::FORBIDDEN, ErrorTemplate::Blank),

            Error::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorTemplate::NotFound(NotFoundTemplate { session: None }),
            ),

            Error::InternalServerError(err) => {
                tracing::error!(%err, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorTemplate::InternalServerError(InternalServerErrorTemplate {
                        session: None,
                    }),
                )
            }
        };

        (status, template).into_response()
    }
}

impl IntoResponse for ErrorTemplate {
    fn into_response(self) -> Response {
        match self {
            ErrorTemplate::InternalServerError(template) => template.into_response(),
            ErrorTemplate::NotFound(template) => template.into_response(),
            ErrorTemplate::Blank => ().into_response(),
        }
    }
}
