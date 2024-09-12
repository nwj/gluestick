use crate::controllers::prelude::*;
use crate::models::session::Session;
use crate::views::AboutTemplate;
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn check() -> StatusCode {
    StatusCode::OK
}

pub async fn about(session: Option<Session>) -> Result<impl IntoResponse> {
    Ok(AboutTemplate { session })
}

pub async fn not_found() -> Result<()> {
    Err(Error::NotFound(None))
}
