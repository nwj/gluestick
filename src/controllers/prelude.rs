use crate::models::prelude::Error as ModelsError;
use crate::models::session::Session;
use crate::views::errors::forbidden::ForbiddenPage;
use crate::views::errors::internal_server_error::InternalServerErrorPage;
use crate::views::errors::not_found::NotFoundPage;
use crate::views::errors::unauthorized::UnauthorizedPage;
use askama::Template;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt::Debug;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid authentication credentials")]
    Unauthorized,

    #[error("invalid authentication credentials")]
    UnauthorizedInline(Box<dyn ErrorTemplate>),

    #[error("insufficient privileges")]
    Forbidden(Option<Session>),

    #[error("resource not found")]
    NotFound(Option<Session>),

    #[error("unprocessable content")]
    Unprocessable(Box<dyn ErrorTemplate>),

    #[error("internal server error: {source}")]
    InternalServerError {
        session: Option<Session>,
        source: Box<dyn std::error::Error>,
    },
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
            Error::Unauthorized => {
                (StatusCode::UNAUTHORIZED, UnauthorizedPage { session: None }).into_response()
            }

            Error::UnauthorizedInline(template) => match template.render_template() {
                Ok(html) => (StatusCode::UNAUTHORIZED, html).into_response(),
                Err(err) => {
                    tracing::error!(%err, "template rendering error");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        InternalServerErrorPage { session: None },
                    )
                        .into_response()
                }
            },

            Error::Forbidden(maybe_session) => (
                StatusCode::FORBIDDEN,
                ForbiddenPage {
                    session: maybe_session,
                },
            )
                .into_response(),

            Error::NotFound(maybe_session) => (
                StatusCode::NOT_FOUND,
                NotFoundPage {
                    session: maybe_session,
                },
            )
                .into_response(),

            Error::Unprocessable(template) => match template.render_template() {
                Ok(html) => (StatusCode::UNPROCESSABLE_ENTITY, html).into_response(),
                Err(err) => {
                    tracing::error!(%err, "template rendering error");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        InternalServerErrorPage { session: None },
                    )
                        .into_response()
                }
            },

            Error::InternalServerError {
                session: maybe_session,
                source,
            } => {
                tracing::error!(%source, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    InternalServerErrorPage {
                        session: maybe_session,
                    },
                )
                    .into_response()
            }
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
        ModelsError::Parse(msg) => Error::Unprocessable(Box::new(f(&msg))),
        e => Error::InternalServerError {
            session,
            source: Box::new(e),
        },
    }
}

pub fn to_unauthorized_error<F, T>(session: Option<Session>, err: ModelsError, f: F) -> Error
where
    F: FnOnce(&str) -> T,
    T: ErrorTemplate + 'static,
{
    match err {
        ModelsError::Parse(msg) => Error::UnauthorizedInline(Box::new(f(&msg))),
        ModelsError::Argon2(e) => Error::UnauthorizedInline(Box::new(f(&e.to_string()))),
        e => Error::InternalServerError {
            session,
            source: Box::new(e),
        },
    }
}
