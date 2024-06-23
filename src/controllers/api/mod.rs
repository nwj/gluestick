use crate::controllers::api::prelude::*;

pub mod pastes;
pub mod prelude;

pub async fn not_found() -> Result<()> {
    Err(self::Error::NotFound)
}
