use crate::controllers::prelude::*;
use crate::models::session::Session;
use crate::views::AboutTemplate;
use axum::response::IntoResponse;

pub mod api;
pub mod api_sessions_controller;
pub mod health_controller;
pub mod pastes_controller;
pub mod prelude;
pub mod sessions_controller;
pub mod users_controller;

pub async fn about(session: Option<Session>) -> Result<impl IntoResponse> {
    Ok(AboutTemplate { session })
}

pub async fn not_found() -> Result<()> {
    Err(Error::NotFound(None))
}
