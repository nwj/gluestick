use crate::controllers::prelude::*;

pub mod api;
pub mod api_sessions;
pub mod health;
pub mod pastes;
pub mod prelude;
pub mod sessions;
pub mod users;

pub async fn not_found() -> Result<()> {
    Err(Error::NotFound)
}
