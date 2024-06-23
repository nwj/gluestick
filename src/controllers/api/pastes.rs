use crate::controllers::api::prelude::*;
use crate::db::Database;
use crate::helpers::pagination::{CursorPaginationParams, CursorPaginationResponse};
use crate::models::api_session::ApiSession;
use crate::models::paste::{Paste, Visibility};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize)]
struct IndexResponse {
    pastes: Vec<Paste>,
    pagination: CursorPaginationResponse,
}

pub async fn index(
    _session: ApiSession,
    State(db): State<Database>,
    pagination_params: Option<Json<CursorPaginationParams>>,
) -> Result<impl IntoResponse> {
    let pagination_params = pagination_params.unwrap_or_default();
    let mut pastes = Paste::cursor_paginated(
        &db,
        pagination_params.limit_with_lookahead(),
        pagination_params.direction(),
        pagination_params.cursor(),
    )
    .await?;
    let pagination = CursorPaginationResponse::new_with_lookahead(&pagination_params, &mut pastes);
    Ok(Json(IndexResponse { pastes, pagination }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePaste {
    filename: String,
    description: String,
    body: String,
    visibility: Visibility,
}

pub async fn create(
    session: ApiSession,
    State(db): State<Database>,
    Json(input): Json<CreatePaste>,
) -> Result<impl IntoResponse> {
    let paste = Paste::new(
        session.user.id,
        input.filename,
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
) -> Result<impl IntoResponse> {
    match Paste::find(&db, id).await? {
        Some(paste) => Ok(Json(paste)),
        None => Err(Error::NotFound),
    }
}

pub async fn show_raw(
    _session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> Result<impl IntoResponse> {
    match Paste::find(&db, id).await? {
        Some(paste) => Ok(paste.body.to_string()),
        None => Err(Error::NotFound),
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePaste {
    filename: Option<String>,
    description: Option<String>,
    body: Option<String>,
}

pub async fn update(
    session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
    Json(input): Json<UpdatePaste>,
) -> Result<impl IntoResponse> {
    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            paste
                .update(&db, input.filename, input.description, input.body)
                .await?;
            Ok(())
        }
        Some(_) => Err(Error::Forbidden),
        None => Err(Error::NotFound),
    }
}

pub async fn destroy(
    session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> Result<impl IntoResponse> {
    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            paste.delete(&db).await?;
            Ok(())
        }
        Some(_) => Err(Error::Forbidden),
        None => Err(Error::NotFound),
    }
}
