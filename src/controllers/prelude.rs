use crate::models::prelude::Error as ModelsError;
use crate::views::{
    ForbiddenTemplate, InternalServerErrorTemplate, NotFoundTemplate, UnauthorizedTemplate,
};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("malformed request")]
    BadRequest(Box<dyn std::error::Error>),

    #[error("invalid authentication credentials")]
    Unauthorized,

    #[error("insufficient privileges")]
    Forbidden,

    #[error("resource not found")]
    NotFound,

    #[allow(clippy::enum_variant_names)]
    #[error("internal server error: {0}")]
    InternalServerError(Box<dyn std::error::Error>),

    #[error("failed validation")]
    Validation(Box<dyn ErrorTemplate>),
}

impl From<ModelsError> for Error {
    fn from(error: ModelsError) -> Self {
        Self::InternalServerError(Box::new(error))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::BadRequest(err) => {
                tracing::error!(%err, "bad request");
                (StatusCode::BAD_REQUEST, err.to_string()).into_response()
            }

            Error::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                UnauthorizedTemplate { session: None },
            )
                .into_response(),

            Error::Forbidden => {
                (StatusCode::FORBIDDEN, ForbiddenTemplate { session: None }).into_response()
            }

            Error::NotFound => {
                (StatusCode::NOT_FOUND, NotFoundTemplate { session: None }).into_response()
            }

            Error::InternalServerError(err) => {
                tracing::error!(%err, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    InternalServerErrorTemplate { session: None },
                )
                    .into_response()
            }

            Error::Validation(template) => match template.render_template() {
                Ok(html) => (StatusCode::OK, html).into_response(),
                Err(err) => {
                    tracing::error!(%err, "template rendering error");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        InternalServerErrorTemplate { session: None },
                    )
                        .into_response()
                }
            },
        }
    }
}

pub fn to_validation_error<F, T>(err: ModelsError, f: F) -> Error
where
    F: FnOnce(&str) -> T,
    T: ErrorTemplate + 'static,
{
    match err {
        ModelsError::Parse(msg) => Error::Validation(Box::new(f(&msg))),
        e => Error::InternalServerError(Box::new(e)),
    }
}

pub trait ErrorTemplate: std::fmt::Debug {
    fn render_template(&self) -> askama::Result<String>;
}
