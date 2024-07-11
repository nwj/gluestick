use crate::models::prelude::Error as ModelsError;
use crate::params::prelude::Error as ParamsError;
use crate::params::prelude::Report;
use crate::views::{InternalServerErrorTemplate, NotFoundTemplate};
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
                (StatusCode::BAD_REQUEST, ()).into_response()
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

            Error::Unauthorized => (StatusCode::UNAUTHORIZED, ()).into_response(),

            Error::Forbidden => (StatusCode::FORBIDDEN, ()).into_response(),

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

pub trait ErrorTemplate: std::fmt::Debug {
    fn render_template(&self) -> askama::Result<String>;
    fn with_report(&mut self, report: Report);
}
