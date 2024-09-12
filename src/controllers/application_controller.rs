use crate::controllers::prelude::*;
use crate::models::session::Session;
use crate::views::IndexTemplate;
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn index(session: Option<Session>) -> Result<impl IntoResponse> {
    Ok(IndexTemplate { session })
}

pub async fn not_found() -> Result<()> {
    Err(Error::NotFound(None))
}
