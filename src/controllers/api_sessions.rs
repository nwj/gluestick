use crate::{
    controllers,
    db::Database,
    models::{
        api_session::{ApiKey, ApiSession},
        session::Session,
    },
    views::api_sessions::CreateApiSessionsTemplate,
};
use axum::{extract::State, response::IntoResponse};

pub async fn create(
    session: Session,
    State(db): State<Database>,
) -> controllers::Result<impl IntoResponse> {
    let api_key = ApiKey::generate();

    ApiSession::new(&api_key, session.user).insert(&db).await?;

    Ok(CreateApiSessionsTemplate { api_key })
}
