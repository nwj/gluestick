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

pub async fn not_found() -> Result<()> {
    Err(self::Error::NotFound)
}

type Result<T, E = Error> = std::result::Result<T, E>;

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

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::BadRequest(err) => {
                tracing::error!(%err, "bad request");
                (StatusCode::BAD_REQUEST, ()).into_response()
            }

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
