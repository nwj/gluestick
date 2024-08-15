use crate::models::prelude::Error as ModelsError;
use crate::params::prelude::Error as ParamsError;
use crate::params::prelude::Report;
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

    #[error("failed validation or verification")]
    Validation(Box<dyn ErrorTemplate>),

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
    Validation2(Box<dyn ErrorTemplate2>),
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

            Error::Validation2(template) => match template.render_template() {
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
        }
    }
}

pub fn handle_params_error(err: ParamsError, mut template: impl ErrorTemplate + 'static) -> Error {
    match err {
        ParamsError::Report(report) => {
            template.with_report(report);
            Error::Validation(Box::new(template))
        }
        ParamsError::Other(err) => Error::InternalServerError(err),
    }
}

pub fn to_validation_error<F, T>(err: ModelsError, f: F) -> Error
where
    F: FnOnce(&str) -> T,
    T: ErrorTemplate2 + 'static,
{
    match err {
        ModelsError::Parse(msg) => Error::Validation2(Box::new(f(&msg))),
        e => Error::InternalServerError(Box::new(e)),
    }
}

pub trait ErrorTemplate: std::fmt::Debug {
    fn render_template(&self) -> askama::Result<String>;
    fn with_report(&mut self, report: Report);
}

pub trait ErrorTemplate2: std::fmt::Debug {
    fn render_template(&self) -> askama::Result<String>;
}
