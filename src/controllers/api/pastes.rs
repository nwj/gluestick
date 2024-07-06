use crate::controllers::api::prelude::*;
use crate::db::Database;
use crate::helpers::pagination::{CursorPaginationParams, CursorPaginationResponse};
use crate::models::api_session::ApiSession;
use crate::models::paste::Paste;
use crate::params::api::pastes::{CreatePasteParams, UpdatePasteParams};
use crate::params::prelude::Validate;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

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
    pagination_params
        .validate()
        .map_err(|e| Error::BadRequest(Box::new(e)))?;

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

pub async fn create(
    session: ApiSession,
    State(db): State<Database>,
    Json(params): Json<CreatePasteParams>,
) -> Result<impl IntoResponse> {
    params
        .validate()
        .map_err(|e| Error::BadRequest(Box::new(e)))?;

    let paste = Paste::new(
        session.user.id,
        params.filename.into(),
        params.description.into(),
        params.body.into(),
        params.visibility.into(),
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

pub async fn update(
    session: ApiSession,
    Path(id): Path<Uuid>,
    State(db): State<Database>,
    Json(params): Json<UpdatePasteParams>,
) -> Result<impl IntoResponse> {
    params
        .validate()
        .map_err(|e| Error::BadRequest(Box::new(e)))?;

    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            paste
                .update(
                    &db,
                    params.filename.map(Into::into),
                    params.description.map(Into::into),
                    params.body.map(Into::into),
                )
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
