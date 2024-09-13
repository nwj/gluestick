use crate::controllers::prelude::*;
use crate::models::session::Session;
use crate::views::index::IndexPage;
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn index(session: Option<Session>) -> Result<impl IntoResponse> {
    Ok(IndexPage { session })
}

pub async fn not_found() -> Result<()> {
    Err(Error::NotFound(None))
}
