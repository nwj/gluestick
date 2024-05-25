use crate::{
    controllers,
    db::Database,
    models::{
        api_session::ApiSession,
        paste::{Paste, Visibility},
    },
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
) -> controllers::api::Result<impl IntoResponse> {
    let pastes = Paste::all(&db).await?;
    Ok(Json(pastes))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePaste {
    title: String,
    description: String,
    body: String,
    visibility: Visibility,
}

pub async fn create(
    session: ApiSession,
    State(db): State<Database>,
    Json(input): Json<CreatePaste>,
) -> controllers::api::Result<impl IntoResponse> {
    let paste = Paste::new(
        session.user.id,
        input.title,
        input.description,
        input.body,
        input.visibility,
    )?;
    let id = paste.id;
    paste.insert(&db).await?;
    Ok(Json(id))
}

pub async fn show(
    _session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> controllers::api::Result<impl IntoResponse> {
    match Paste::find(&db, id).await? {
        Some(paste) => Ok(Json(paste)),
        None => Err(controllers::api::Error::NotFound),
    }
}

pub async fn show_raw(
    _session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> controllers::api::Result<impl IntoResponse> {
    match Paste::find(&db, id).await? {
        Some(paste) => Ok(paste.body.to_string()),
        None => Err(controllers::api::Error::NotFound),
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePaste {
    title: Option<String>,
    description: Option<String>,
    body: Option<String>,
}

pub async fn update(
    session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
    Json(input): Json<UpdatePaste>,
) -> controllers::api::Result<impl IntoResponse> {
    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            paste
                .update(&db, input.title, input.description, input.body)
                .await?;
            Ok(())
        }
        Some(_) => Err(controllers::api::Error::Forbidden),
        None => Err(controllers::api::Error::NotFound),
    }
}

pub async fn destroy(
    session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> controllers::api::Result<impl IntoResponse> {
    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            paste.delete(&db).await?;
            Ok(())
        }
        Some(_) => Err(controllers::api::Error::Forbidden),
        None => Err(controllers::api::Error::NotFound),
    }
}
