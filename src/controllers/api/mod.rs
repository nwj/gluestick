use crate::controllers::api::prelude::*;

pub mod pastes_controller;
pub mod prelude;

pub async fn not_found() -> Result<()> {
    Err(self::Error::NotFound)
}
