use crate::controllers::prelude::*;
use crate::models::session::Session;
use crate::views::AboutTemplate;
use axum::response::IntoResponse;

pub mod api;
pub mod api_sessions;
pub mod health;
pub mod pastes;
pub mod prelude;
pub mod sessions;
pub mod users;

pub async fn about(session: Option<Session>) -> Result<impl IntoResponse> {
    Ok(AboutTemplate { session })
}

pub async fn not_found() -> Result<()> {
    Err(Error::NotFound(None))
}
