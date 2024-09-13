use crate::controllers::prelude::*;
use crate::db::Database;
use crate::models::api_session::ApiKey;
use crate::models::session::Session;
use crate::views::api_sessions::create::CreatePage;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

pub async fn create(session: Session, State(db): State<Database>) -> Result<impl IntoResponse> {
    let (unhashed_key, api_key) = ApiKey::new(session.user.id);
    let api_key_id = api_key.id;

    api_key.insert(&db).await?;

    Ok(CreatePage {
        unhashed_key,
        api_key_id,
    })
}

pub async fn destroy(
    session: Session,
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound(Some(session.clone())))?;

    let api_key = ApiKey::find_scoped_by_user_id(&db, id, session.user.id)
        .await?
        .ok_or(Error::NotFound(Some(session)))?;
    api_key.delete(&db).await?;

    Ok(())
}
