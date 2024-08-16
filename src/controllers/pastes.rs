use crate::controllers::prelude::*;
use crate::db::Database;
use crate::helpers::pagination::{CursorPaginationParams, CursorPaginationResponse};
use crate::models::paste::{Body, Description, Filename, Paste, Visibility};
use crate::models::prelude::Error as ModelsError;
use crate::models::session::Session;
use crate::models::user::{User, Username};
use crate::views::pastes::{
    EditPastesFormPartial, EditPastesTemplate, IndexPastesTemplate, NewPastesFormPartial,
    NewPastesTemplate, ShowPastesTemplate,
};
use axum::extract::{Form, Path, Query, State};
use axum::http::{header::HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use serde::Deserialize;
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
    NewPastesTemplate::from(session)
}

#[derive(Clone, Deserialize)]
pub struct CreatePasteParams {
    pub filename: String,
    pub description: String,
    pub body: String,
    pub visibility: String,
}

pub async fn create(
    session: Session,
    State(db): State<Database>,
    Form(params): Form<CreatePasteParams>,
) -> Result<impl IntoResponse> {
    let user_id = session.user.id;
    let username = session.user.username.clone();
    let mut error_template: NewPastesFormPartial = (username.clone(), params.clone()).into();

    let filename_result = Filename::try_from(&params.filename);
    if let Err(ModelsError::Parse(ref msg)) = filename_result {
        error_template.filename_error_message = Some(msg.into());
    }
    let description_result = Description::try_from(&params.description);
    if let Err(ModelsError::Parse(ref msg)) = description_result {
        error_template.description_error_message = Some(msg.into());
    }
    let body_result = Body::try_from(&params.body);
    if let Err(ModelsError::Parse(ref msg)) = body_result {
        error_template.body_error_message = Some(msg.into());
    }
    let visibility =
        Visibility::try_from(&params.visibility).map_err(|e| Error::BadRequest(Box::new(e)))?;

    if error_template.filename_error_message.is_some()
        || error_template.description_error_message.is_some()
        || error_template.body_error_message.is_some()
    {
        return Err(Error::Validation(Box::new(error_template)));
    }

    let (filename, description, body) = (filename_result?, description_result?, body_result?);
    let paste = Paste::new(user_id, filename, description, body, visibility)?;
    let paste_id = paste.id;
    paste.insert(&db).await?;

    let mut response = HeaderMap::new();
    response.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/{username}/{paste_id}"))
            .map_err(|e| Error::InternalServerError(Box::new(e)))?,
    );

    Ok(response)
}

pub async fn show(
    session: Option<Session>,
    State(db): State<Database>,
    Path((username, id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
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
        EditPastesTemplate::from((session, paste)),
    ))
}

#[derive(Clone, Deserialize)]
pub struct UpdatePasteParams {
    pub filename: String,
    pub description: String,
    pub body: String,
}

pub async fn update(
    session: Session,
    State(db): State<Database>,
    Path((username, paste_id)): Path<(String, String)>,
    Form(params): Form<UpdatePasteParams>,
) -> Result<impl IntoResponse> {
    let paste_id = Uuid::try_parse(&paste_id).map_err(|_| Error::NotFound)?;
    let username = Username::try_from(&username).map_err(|_| Error::NotFound)?;

    let user = User::find_by_username(&db, username.clone())
        .await?
        .ok_or(Error::NotFound)?;

    if session.user != user {
        return Err(Error::Forbidden);
    }

    let paste = Paste::find_scoped_by_user_id(&db, paste_id, session.user.id)
        .await?
        .ok_or(Error::NotFound)?;

    let mut response = HeaderMap::new();
    response.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/{username}/{}", &paste.id))
            .map_err(|e| Error::InternalServerError(Box::new(e)))?,
    );

    let mut error_template = EditPastesFormPartial::from((username, paste_id, params.clone()));

    let filename_result = Filename::try_from(&params.filename);
    if let Err(ModelsError::Parse(ref msg)) = filename_result {
        error_template.filename_error_message = Some(msg.into());
    }
    let description_result = Description::try_from(&params.description);
    if let Err(ModelsError::Parse(ref msg)) = description_result {
        error_template.description_error_message = Some(msg.into());
    }
    let body_result = Body::try_from(&params.body);
    if let Err(ModelsError::Parse(ref msg)) = body_result {
        error_template.body_error_message = Some(msg.into());
    }

    if error_template.filename_error_message.is_some()
        || error_template.description_error_message.is_some()
        || error_template.body_error_message.is_some()
    {
        return Err(Error::Validation(Box::new(error_template)));
    }

    let (filename, description, body) = (filename_result?, description_result?, body_result?);
    paste
        .update(&db, Some(filename), Some(description), Some(body))
        .await?;

    Ok(response)
}

pub async fn destroy(
    session: Session,
    State(db): State<Database>,
    Path((username, id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
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
