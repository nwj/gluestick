use crate::{
    controllers,
    db::Database,
    models::{
        api_session::ApiSession,
        paste::{Paste, Visibility},
    },
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
    visibility: Visibility,
}

pub async fn create(
    session: ApiSession,
    State(db): State<Database>,
    Json(input): Json<CreatePaste>,
) -> Result<impl IntoResponse, controllers::api::Error> {
    input.validate()?;
    let paste = Paste::new(
        session.user.id,
        input.title,
        input.description,
        input.body,
        input.visibility,
    );
    let id = paste.id;
    paste
        .insert(&db)
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

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePaste {
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    title: Option<String>,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    description: Option<String>,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    body: Option<String>,
}

pub async fn update(
    session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
    Json(input): Json<UpdatePaste>,
) -> Result<impl IntoResponse, controllers::api::Error> {
    let optional_paste = Paste::find(&db, id)
        .await
        .map_err(|e| controllers::api::Error::InternalServerError(Box::new(e)))?;

    match optional_paste {
        Some(mut paste) if paste.user_id == session.user.id => {
            if let Some(title) = input.title {
                paste.title = title;
            }
            if let Some(description) = input.description {
                paste.description = description;
            }
            if let Some(body) = input.body {
                paste.body = body;
            }

            paste
                .update(&db)
                .await
                .map_err(|e| controllers::api::Error::InternalServerError(Box::new(e)))?;
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
) -> Result<impl IntoResponse, controllers::api::Error> {
    let optional_paste = Paste::find(&db, id)
        .await
        .map_err(|e| controllers::api::Error::InternalServerError(Box::new(e)))?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            paste
                .delete(&db)
                .await
                .map_err(|e| controllers::api::Error::InternalServerError(Box::new(e)))?;
            Ok(())
        }
        Some(_) => Err(controllers::api::Error::Forbidden),
        None => Err(controllers::api::Error::NotFound),
    }
}
