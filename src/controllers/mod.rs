use crate::views::NotFoundTemplate;
use axum::{http::StatusCode, response::IntoResponse};

pub mod pastes;

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, NotFoundTemplate {})
}
