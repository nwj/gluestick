use crate::{
    controllers,
    db::Database,
    models::{api_session::ApiSession, paste::Paste},
    validators,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

pub async fn index(
    _session: ApiSession,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::api::Error> {
    let pastes = Paste::all(&db)
        .await
        .map_err(|e| controllers::api::Error::InternalServerError(Box::new(e)))?;
    Ok(Json(pastes))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePaste {
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    title: String,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    description: String,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    body: String,
}

pub async fn create(
    session: ApiSession,
    State(db): State<Database>,
    Json(input): Json<CreatePaste>,
) -> Result<impl IntoResponse, controllers::api::Error> {
    input.validate()?;
    let id = Paste::insert(
        &db,
        session.user.id,
        input.title,
        input.description,
        input.body,
    )
    .await
    .map_err(|e| controllers::api::Error::InternalServerError(Box::new(e)))?;
    Ok(Json(id))
}

pub async fn show(
    _session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::api::Error> {
    match Paste::find(&db, id)
        .await
        .map_err(|e| controllers::api::Error::InternalServerError(Box::new(e)))?
    {
        Some(paste) => Ok(Json(paste)),
        None => Err(controllers::api::Error::NotFound),
    }
}

pub async fn destroy(
    _session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::api::Error> {
    match Paste::delete(&db, id)
        .await
        .map_err(|e| controllers::api::Error::InternalServerError(Box::new(e)))?
    {
        0 => Err(controllers::api::Error::NotFound),
        _ => Ok(()),
    }
}
