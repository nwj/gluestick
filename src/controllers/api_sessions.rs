use crate::controllers::prelude::*;
use crate::db::Database;
use crate::models::api_session::ApiKey;
use crate::models::session::Session;
use crate::views::api_sessions::CreateApiSessionsTemplate;
use axum::{extract::State, response::IntoResponse};

pub async fn create(session: Session, State(db): State<Database>) -> Result<impl IntoResponse> {
    let (unhashed_key, api_key) = ApiKey::new(session.user.id);
    api_key.insert(&db).await?;
    Ok(CreateApiSessionsTemplate { unhashed_key })
}
