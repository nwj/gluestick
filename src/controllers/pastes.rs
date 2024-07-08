use crate::controllers::prelude::*;
use crate::db::Database;
use crate::helpers::pagination::{CursorPaginationParams, CursorPaginationResponse};
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::params::pastes::{CreatePasteParams, UpdatePasteParams};
use crate::params::prelude::Validate;
use crate::views::pastes::{
    EditPastesTemplate, IndexPastesTemplate, NewPastesTemplate, ShowPastesTemplate,
};
use axum::extract::{Form, Path, Query, State};
use axum::http::{header::HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect};
use uuid::Uuid;

pub async fn index(
    session: Option<Session>,
    Query(pagination_params): Query<CursorPaginationParams>,
    State(db): State<Database>,
) -> Result<impl IntoResponse> {
    pagination_params
        .validate()
        .map_err(|e| Error::BadRequest(Box::new(e)))?;

    let mut pairs = Paste::cursor_paginated_with_username(
        &db,
        pagination_params.limit_with_lookahead(),
        pagination_params.direction(),
        pagination_params.cursor(),
    )
    .await?;
    let pagination_response =
        CursorPaginationResponse::new_with_lookahead(&pagination_params, &mut pairs);
    let mut triples = Vec::new();
    for (paste, username) in pairs {
        let optional_html = paste
            .syntax_highlight(&db) // This is an n+1 query, but it's fine because our cache is SQLite.
            .await?;
        triples.push((paste, username, optional_html));
    }
    Ok(IndexPastesTemplate {
        session,
        paste_username_html_triples: triples,
        pagination: pagination_response,
    })
}

pub async fn new(session: Session) -> NewPastesTemplate {
    let session = Some(session);
    NewPastesTemplate::from_session(session)
}

pub async fn create(
    session: Session,
    State(db): State<Database>,
    Form(params): Form<CreatePasteParams>,
) -> Result<impl IntoResponse> {
    let user_id = session.user.id;
    let error_template = NewPastesTemplate::from_session_and_params(Some(session), params.clone());
    params
        .validate()
        .map_err(|e| handle_params_error(e, error_template))?;

    let paste = Paste::new(
        user_id,
        params.filename.into(),
        params.description.into(),
        params.body.into(),
        params.visibility.into(),
    )?;
    let id = paste.id;
    paste.insert(&db).await?;

    Ok(Redirect::to(format!("/pastes/{id}").as_str()).into_response())
}

pub async fn show(
    session: Option<Session>,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    match Paste::find_with_username(&db, id).await? {
        Some((paste, username)) => {
            let syntax_highlighted_html = paste.syntax_highlight(&db).await?;
            Ok((
                StatusCode::OK,
                ShowPastesTemplate {
                    session,
                    paste,
                    username,
                    syntax_highlighted_html,
                },
            ))
        }
        None => Err(Error::NotFound),
    }
}

pub async fn show_raw(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    match Paste::find(&db, id).await? {
        Some(paste) => Ok((StatusCode::OK, paste.body.to_string())),
        None => Err(Error::NotFound),
    }
}

pub async fn download(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    match Paste::find(&db, id).await? {
        Some(paste) => Ok((
            StatusCode::OK,
            [(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", paste.filename),
            )],
            paste.body.to_string(),
        )),
        None => Err(Error::NotFound),
    }
}

pub async fn edit(
    session: Session,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            let response = EditPastesTemplate {
                session: Some(session),
                paste_id: paste.id,
                filename: paste.filename.into(),
                description: paste.description.into(),
                body: paste.body.into(),
                ..Default::default()
            };
            Ok(response)
        }
        Some(_) => Err(Error::Forbidden),
        None => Err(Error::NotFound),
    }
}

pub async fn update(
    session: Session,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
    Form(params): Form<UpdatePasteParams>,
) -> Result<impl IntoResponse> {
    let user_id = session.user.id;
    let error_template = EditPastesTemplate::from_session_and_params(Some(session), params.clone());
    params
        .validate()
        .map_err(|e| handle_params_error(e, error_template))?;

    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == user_id => {
            let mut response = HeaderMap::new();
            response.insert(
                "HX-Redirect",
                HeaderValue::from_str(&format!("/pastes/{}", &paste.id))
                    .map_err(|e| Error::InternalServerError(Box::new(e)))?,
            );

            paste
                .update(
                    &db,
                    Some(params.filename.into()),
                    Some(params.description.into()),
                    Some(params.body.into()),
                )
                .await?;

            Ok(response)
        }
        Some(_) => Err(Error::Forbidden),
        None => Err(Error::NotFound),
    }
}

pub async fn destroy(
    session: Session,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            paste.delete(&db).await?;

            let mut response = HeaderMap::new();
            response.insert("HX-Redirect", HeaderValue::from_static("/pastes"));
            Ok(response)
        }
        Some(_) => Err(Error::Forbidden),
        None => Err(Error::NotFound),
    }
}
