use crate::{
    controllers,
    db::Database,
    models::{
        paste::{Paste, Visibility},
        session::Session,
    },
    views::pastes::{
        EditPastesTemplate, IndexPastesTemplate, NewPastesTemplate, ShowPastesTemplate,
    },
};
use axum::{
    extract::{Form, Path, State},
    http::{header::HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

pub async fn index(
    session: Option<Session>,
    State(db): State<Database>,
) -> controllers::Result<impl IntoResponse> {
    let paste_username_pairs = Paste::all_with_usernames(&db).await?;
    Ok(IndexPastesTemplate {
        session,
        paste_username_pairs,
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
    match Paste::find_with_username(&db, id).await? {
        Some((paste, username)) => Ok((
            StatusCode::OK,
            ShowPastesTemplate {
                session,
                paste,
                username,
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
