use crate::controllers::prelude::*;
use crate::db::Database;
use crate::models::api_session::{ApiKey, ApiSession};
use crate::models::session::Session;
use crate::views::api_sessions::CreateApiSessionsTemplate;
use axum::{extract::State, response::IntoResponse};

pub async fn create(session: Session, State(db): State<Database>) -> Result<impl IntoResponse> {
    let api_key = ApiKey::generate();
    ApiSession::new(&api_key, session.user).insert(&db).await?;
    Ok(CreateApiSessionsTemplate { api_key })
}
