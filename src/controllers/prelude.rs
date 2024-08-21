use crate::models::prelude::Error as ModelsError;
use crate::models::session::Session;
use crate::views::{
    ForbiddenTemplate, InternalServerErrorTemplate, NotFoundTemplate, UnauthorizedTemplate,
};
use askama::Template;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt::Debug;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("malformed request")]
    BadRequest(Box<dyn std::error::Error>),

    #[error("invalid authentication credentials")]
    Unauthorized,

    #[error("insufficient privileges")]
    Forbidden(Option<Session>),

    #[error("resource not found")]
    NotFound(Option<Session>),

    #[allow(clippy::enum_variant_names)]
    #[error("internal server error: {source}")]
    InternalServerError {
        session: Option<Session>,
        source: Box<dyn std::error::Error>,
    },

    #[error("failed validation")]
    Validation(Box<dyn ErrorTemplate>),
}

impl From<ModelsError> for Error {
    fn from(error: ModelsError) -> Self {
        Self::InternalServerError {
            session: None,
            source: Box::new(error),
        }
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

            Error::Forbidden(maybe_session) => (
                StatusCode::FORBIDDEN,
                ForbiddenTemplate {
                    session: maybe_session,
                },
            )
                .into_response(),

            Error::NotFound(maybe_session) => (
                StatusCode::NOT_FOUND,
                NotFoundTemplate {
                    session: maybe_session,
                },
            )
                .into_response(),

            Error::InternalServerError {
                session: maybe_session,
                source,
            } => {
                tracing::error!(%source, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    InternalServerErrorTemplate {
                        session: maybe_session,
                    },
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

pub trait ErrorTemplate: Debug {
    fn render_template(&self) -> askama::Result<String>;
}

impl<T: Template + Debug> ErrorTemplate for T {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
    }
}

pub fn to_validation_error<F, T>(session: Option<Session>, err: ModelsError, f: F) -> Error
where
    F: FnOnce(&str) -> T,
    T: ErrorTemplate + 'static,
{
    match err {
        ModelsError::Parse(msg) => Error::Validation(Box::new(f(&msg))),
        e => Error::InternalServerError {
            session,
            source: Box::new(e),
        },
    }
}
