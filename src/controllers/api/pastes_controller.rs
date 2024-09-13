use crate::controllers::api::prelude::*;
use crate::db::Database;
use crate::helpers::pagination_helper::{CursorPaginationParams, CursorPaginationResponse};
use crate::models::api_session::ApiSession;
use crate::models::paste::{Body, Description, Filename, Paste, Visibility};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Deserialize)]
pub struct CreatePasteParams {
    pub filename: String,
    pub description: String,
    pub body: String,
    pub visibility: String,
}

pub async fn create(
    session: ApiSession,
    State(db): State<Database>,
    Json(params): Json<CreatePasteParams>,
) -> Result<impl IntoResponse> {
    let filename =
        Filename::try_from(&params.filename).map_err(|e| Error::Unprocessable(Box::new(e)))?;
    let description = Description::try_from(&params.description)
        .map_err(|e| Error::Unprocessable(Box::new(e)))?;
    let body = Body::try_from(&params.body).map_err(|e| Error::Unprocessable(Box::new(e)))?;
    let visibility =
        Visibility::try_from(&params.visibility).map_err(|e| Error::Unprocessable(Box::new(e)))?;

    let paste = Paste::new(session.user.id, filename, description, body, visibility)?;
    let id = paste.id;
    paste.insert(&db).await?;
    Ok(Json(id))
}

pub async fn show(
    _session: ApiSession,
    Path(id): Path<String>,
    State(db): State<Database>,
) -> Result<impl IntoResponse> {
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound)?;
    match Paste::find(&db, id).await? {
        Some(paste) => Ok(Json(paste)),
        None => Err(Error::NotFound),
    }
}

pub async fn show_raw(
    _session: ApiSession,
    Path(id): Path<String>,
    State(db): State<Database>,
) -> Result<impl IntoResponse> {
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound)?;
    match Paste::find(&db, id).await? {
        Some(paste) => Ok(paste.body.to_string()),
        None => Err(Error::NotFound),
    }
}

#[derive(Clone, Deserialize)]
pub struct UpdatePasteParams {
    pub filename: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    pub visibility: Option<String>,
}

pub async fn update(
    session: ApiSession,
    Path(id): Path<String>,
    State(db): State<Database>,
    Json(params): Json<UpdatePasteParams>,
) -> Result<impl IntoResponse> {
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound)?;
    let filename = match params.filename {
        Some(filename) => {
            Some(Filename::try_from(&filename).map_err(|e| Error::Unprocessable(Box::new(e)))?)
        }
        None => None,
    };
    let description = match params.description {
        Some(description) => Some(
            Description::try_from(&description).map_err(|e| Error::Unprocessable(Box::new(e)))?,
        ),
        None => None,
    };
    let body = match params.body {
        Some(body) => Some(Body::try_from(&body).map_err(|e| Error::Unprocessable(Box::new(e)))?),
        None => None,
    };
    let visibility = match params.visibility {
        Some(visibility) => {
            Some(Visibility::try_from(&visibility).map_err(|e| Error::Unprocessable(Box::new(e)))?)
        }
        None => None,
    };

    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            // Once a paste is public, we don't let people update it back to secret because the
            // paste could have been indexed (or otherwise seen/recorded by someone) and we don't
            // want to give the impression that we can somehow undo the paste's public disclosure
            if paste.visibility.is_public()
                && visibility.as_ref().map_or(false, Visibility::is_secret)
            {
                let e = ValidationError("Cannot change from public to secret visibility".into());
                Err(Error::Unprocessable(Box::new(e)))
            } else {
                paste
                    .update(&db, filename, description, body, visibility)
                    .await?;
                Ok(())
            }
        }
        Some(_) => Err(Error::Forbidden),
        None => Err(Error::NotFound),
    }
}

pub async fn destroy(
    session: ApiSession,
    Path(id): Path<String>,
    State(db): State<Database>,
) -> Result<impl IntoResponse> {
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound)?;
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
