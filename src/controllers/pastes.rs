use crate::{
    controllers,
    db::Database,
    models::{
        paste::{Paste, Visibility},
        session::Session,
    },
    validators,
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
) -> Result<impl IntoResponse, controllers::Error> {
    let pastes = Paste::all(&db)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;
    Ok(IndexPastesTemplate { session, pastes })
}

pub async fn new(session: Session) -> NewPastesTemplate {
    let session = Some(session);
    NewPastesTemplate { session }
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreatePaste {
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub title: String,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub description: String,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub body: String,
    pub visibility: Visibility,
}

pub async fn create(
    session: Session,
    State(db): State<Database>,
    Form(input): Form<CreatePaste>,
) -> Result<impl IntoResponse, controllers::Error> {
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
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

    Ok(Redirect::to(format!("/pastes/{id}").as_str()).into_response())
}

pub async fn show(
    session: Option<Session>,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, controllers::Error> {
    match Paste::find(&db, id)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?
    {
        Some(paste) => Ok((StatusCode::OK, ShowPastesTemplate { session, paste })),
        None => Err(controllers::Error::NotFound),
    }
}

pub async fn edit(
    session: Session,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, controllers::Error> {
    let optional_paste = Paste::find(&db, id)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

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
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub title: String,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub description: String,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub body: String,
}

pub async fn update(
    session: Session,
    State(db): State<Database>,
    Path(id): Path<Uuid>,
    Form(input): Form<UpdatePaste>,
) -> Result<impl IntoResponse, controllers::Error> {
    let optional_paste = Paste::find(&db, id)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

    match optional_paste {
        Some(mut paste) if paste.user_id == session.user.id => {
            paste.title = input.title;
            paste.description = input.description;
            paste.body = input.body;

            let mut response = HeaderMap::new();
            response.insert(
                "HX-Redirect",
                HeaderValue::from_str(&format!("/pastes/{}", &paste.id)).unwrap(),
            );

            paste
                .update(&db)
                .await
                .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

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
) -> Result<impl IntoResponse, controllers::Error> {
    let optional_paste = Paste::find(&db, id)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

    match optional_paste {
        Some(paste) if paste.user_id == session.user.id => {
            paste
                .delete(&db)
                .await
                .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

            let mut response = HeaderMap::new();
            response.insert("HX-Redirect", HeaderValue::from_static("/pastes"));
            Ok(response)
        }
        Some(_) => Err(controllers::Error::Forbidden),
        None => Err(controllers::Error::NotFound),
    }
}
