use crate::{
    controllers,
    db::Database,
    helpers::pagination::{CursorPaginationParams, CursorPaginationResponse},
    models::{
        paste::{Paste, Visibility},
        session::Session,
    },
    views::pastes::{
        EditPastesTemplate, IndexPastesTemplate, NewPastesTemplate, ShowPastesTemplate,
    },
};
use axum::{
    extract::{Form, Path, Query, State},
    http::{header::HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

pub async fn index(
    session: Option<Session>,
    Query(pagination_params): Query<CursorPaginationParams>,
    State(db): State<Database>,
) -> controllers::Result<impl IntoResponse> {
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
        // This is an n+1 query, but it's fine because our cache is SQLite.
        let optional_html = paste
            .to_syntax_highlighted_html_with_cache_attempt(&db)
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
    NewPastesTemplate { session }
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreatePaste {
    pub filename: String,
    pub description: String,
    pub body: String,
    pub visibility: Visibility,
}

pub async fn create(
    session: Session,
    State(db): State<Database>,
    Form(input): Form<CreatePaste>,
) -> controllers::Result<impl IntoResponse> {
    let paste = Paste::new(
        session.user.id,
        input.filename,
        input.description,
        input.body,
        input.visibility,
    )?;
    let id = paste.id;
    paste.insert(&db).await?;

    Ok(Redirect::to(format!("/pastes/{id}").as_str()).into_response())
}

pub async fn show(
    session: Option<Session>,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> controllers::Result<impl IntoResponse> {
    match Paste::find_with_username_and_syntax_highlighted_html(&db, id).await? {
        Some((paste, username, syntax_highlighted_html)) => Ok((
            StatusCode::OK,
            ShowPastesTemplate {
                session,
                paste,
                username,
                syntax_highlighted_html,
            },
        )),
        None => Err(controllers::Error::NotFound),
    }
}

pub async fn show_raw(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> controllers::Result<impl IntoResponse> {
    match Paste::find(&db, id).await? {
        Some(paste) => Ok((StatusCode::OK, paste.body.to_string())),
        None => Err(controllers::Error::NotFound),
    }
}

pub async fn download(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> controllers::Result<impl IntoResponse> {
    match Paste::find(&db, id).await? {
        Some(paste) => Ok((
            StatusCode::OK,
            [(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", paste.filename),
            )],
            paste.body.to_string(),
        )),
        None => Err(controllers::Error::NotFound),
    }
}

pub async fn edit(
    session: Session,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> controllers::Result<impl IntoResponse> {
    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            let response = EditPastesTemplate {
                session: Some(session),
                paste,
            };
            Ok(response)
        }
        Some(_) => Err(controllers::Error::Forbidden),
        None => Err(controllers::Error::NotFound),
    }
}

#[derive(Deserialize, Debug, Validate)]
pub struct UpdatePaste {
    pub filename: String,
    pub description: Option<String>,
    pub body: String,
}

pub async fn update(
    session: Session,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
    Form(input): Form<UpdatePaste>,
) -> controllers::Result<impl IntoResponse> {
    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            let mut response = HeaderMap::new();
            response.insert(
                "HX-Redirect",
                HeaderValue::from_str(&format!("/pastes/{}", &paste.id)).unwrap(),
            );

            paste
                .update(
                    &db,
                    Some(input.filename),
                    input.description,
                    Some(input.body),
                )
                .await?;

            Ok(response)
        }
        Some(_) => Err(controllers::Error::Forbidden),
        None => Err(controllers::Error::NotFound),
    }
}

pub async fn destroy(
    session: Session,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> controllers::Result<impl IntoResponse> {
    let optional_paste = Paste::find(&db, id).await?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            paste.delete(&db).await?;

            let mut response = HeaderMap::new();
            response.insert("HX-Redirect", HeaderValue::from_static("/pastes"));
            Ok(response)
        }
        Some(_) => Err(controllers::Error::Forbidden),
        None => Err(controllers::Error::NotFound),
    }
}
