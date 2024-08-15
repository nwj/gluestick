use crate::controllers::prelude::*;
use crate::db::Database;
use crate::helpers::pagination::{CursorPaginationParams, CursorPaginationResponse};
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::{User, Username};
use crate::params::pastes::{CreatePasteParams, UpdatePasteParams};
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
    NewPastesTemplate::from_session(session)
}

pub async fn create(
    session: Session,
    State(db): State<Database>,
    Form(params): Form<CreatePasteParams>,
) -> Result<impl IntoResponse> {
    let user_id = session.user.id;
    let username = session.user.username.clone();
    let error_template = NewPastesTemplate::from_session_and_params(session, params.clone());
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

    Ok(Redirect::to(format!("/{username}/{id}").as_str()).into_response())
}

pub async fn show(
    session: Option<Session>,
    State(db): State<Database>,
    Path((username, id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // Manually parse id here so that we can render NotFound (rather than BadRequest if we let
    // Axum + Serde automatically deserialize to Uuid)
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound)?;
    let username = Username::try_from(&username).map_err(|_| Error::NotFound)?;

    let user = User::find_by_username(&db, username)
        .await?
        .ok_or(Error::NotFound)?;
    let paste = Paste::find_scoped_by_user_id(&db, id, user.id)
        .await?
        .ok_or(Error::NotFound)?;
    let syntax_highlighted_html = paste.syntax_highlight(&db).await?;

    let mut headers = HeaderMap::new();
    if paste.visibility.is_secret() {
        headers.insert("X-Robots-Tag", HeaderValue::from_static("noindex"));
    }

    Ok((
        StatusCode::OK,
        headers,
        ShowPastesTemplate {
            session,
            paste,
            username: user.username,
            syntax_highlighted_html,
        },
    ))
}

pub async fn show_raw(
    State(db): State<Database>,
    Path((username, id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // Manually parse id here so that we can render NotFound (rather than BadRequest if we let
    // Axum + Serde automatically deserialize to Uuid)
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound)?;
    let username = Username::try_from(&username).map_err(|_| Error::NotFound)?;

    let user = User::find_by_username(&db, username)
        .await?
        .ok_or(Error::NotFound)?;
    let paste = Paste::find_scoped_by_user_id(&db, id, user.id)
        .await?
        .ok_or(Error::NotFound)?;

    let mut headers = HeaderMap::new();
    if paste.visibility.is_secret() {
        headers.insert("X-Robots-Tag", HeaderValue::from_static("noindex"));
    }

    Ok((StatusCode::OK, headers, paste.body.to_string()))
}

pub async fn download(
    State(db): State<Database>,
    Path((username, id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // Manually parse id here so that we can render NotFound (rather than BadRequest if we let
    // Axum + Serde automatically deserialize to Uuid)
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound)?;
    let username = Username::try_from(&username).map_err(|_| Error::NotFound)?;

    let user = User::find_by_username(&db, username)
        .await?
        .ok_or(Error::NotFound)?;
    let paste = Paste::find_scoped_by_user_id(&db, id, user.id)
        .await?
        .ok_or(Error::NotFound)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", paste.filename))
            .map_err(|e| Error::InternalServerError(Box::new(e)))?,
    );
    if paste.visibility.is_secret() {
        headers.insert("X-Robots-Tag", HeaderValue::from_static("noindex"));
    }

    Ok((StatusCode::OK, headers, paste.body.to_string()))
}

pub async fn edit(
    session: Session,
    State(db): State<Database>,
    Path((username, id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // Manually parse id here so that we can render NotFound (rather than BadRequest if we let
    // Axum + Serde automatically deserialize to Uuid)
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound)?;
    let username = Username::try_from(&username).map_err(|_| Error::NotFound)?;

    let user = User::find_by_username(&db, username)
        .await?
        .ok_or(Error::NotFound)?;
    if session.user != user {
        return Err(Error::Forbidden);
    }
    let paste = Paste::find_scoped_by_user_id(&db, id, session.user.id)
        .await?
        .ok_or(Error::NotFound)?;

    let mut headers = HeaderMap::new();
    if paste.visibility.is_secret() {
        headers.insert("X-Robots-Tag", HeaderValue::from_static("noindex"));
    }

    Ok((
        StatusCode::OK,
        headers,
        EditPastesTemplate {
            session: Some(session),
            paste_id: paste.id,
            filename: paste.filename.into(),
            description: paste.description.into(),
            body: paste.body.into(),
            ..Default::default()
        },
    ))
}

pub async fn update(
    session: Session,
    State(db): State<Database>,
    Path((username, id)): Path<(String, String)>,
    Form(params): Form<UpdatePasteParams>,
) -> Result<impl IntoResponse> {
    // Manually parse id here so that we can render NotFound (rather than BadRequest if we let
    // Axum + Serde automatically deserialize to Uuid)
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound)?;
    let username = Username::try_from(&username).map_err(|_| Error::NotFound)?;

    let user = User::find_by_username(&db, username)
        .await?
        .ok_or(Error::NotFound)?;
    if session.user != user {
        return Err(Error::Forbidden);
    }
    let username = session.user.username.clone();
    let user_id = session.user.id;

    let error_template = EditPastesTemplate::from_session_and_params(Some(session), params.clone());
    params
        .validate()
        .map_err(|e| handle_params_error(e, error_template))?;

    let paste = Paste::find_scoped_by_user_id(&db, id, user_id)
        .await?
        .ok_or(Error::NotFound)?;

    let mut response = HeaderMap::new();
    response.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/{username}/{}", &paste.id))
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

pub async fn destroy(
    session: Session,
    State(db): State<Database>,
    Path((username, id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // Manually parse id here so that we can render NotFound (rather than BadRequest if we let
    // Axum + Serde automatically deserialize to Uuid)
    let id = Uuid::try_parse(&id).map_err(|_| Error::NotFound)?;
    let username = Username::try_from(&username).map_err(|_| Error::NotFound)?;

    let user = User::find_by_username(&db, username)
        .await?
        .ok_or(Error::NotFound)?;
    if session.user != user {
        return Err(Error::Forbidden);
    }

    let paste = Paste::find_scoped_by_user_id(&db, id, session.user.id)
        .await?
        .ok_or(Error::NotFound)?;
    paste.delete(&db).await?;

    let mut response = HeaderMap::new();
    response.insert("HX-Redirect", HeaderValue::from_static("/pastes"));
    Ok(response)
}
