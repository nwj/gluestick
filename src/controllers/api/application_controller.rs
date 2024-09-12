use crate::controllers::api::prelude::*;

pub async fn not_found() -> Result<()> {
    Err(self::Error::NotFound)
}
