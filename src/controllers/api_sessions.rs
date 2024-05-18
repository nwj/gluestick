use crate::{
    controllers,
    db::Database,
    models::{
        api_session::{ApiKey, ApiSession},
        session::Session,
    },
    views::api_sessions::ApiSessionsCreateTemplate,
};
use axum::{extract::State, response::IntoResponse};

pub async fn create(
    session: Session,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::Error> {
    let api_key = ApiKey::generate();

    ApiSession {
        api_key: api_key.clone(),
        user: session.user,
    }
    .insert(&db)
    .await
    .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

    Ok(ApiSessionsCreateTemplate { api_key })
}
