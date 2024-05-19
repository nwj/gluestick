use crate::{
    controllers,
    db::Database,
    models::{paste::Paste, session::Session},
    validators,
    views::pastes::{IndexPastesTemplate, NewPastesTemplate, ShowPastesTemplate},
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
pub struct CreateFormInput {
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub title: String,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub description: String,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub body: String,
}

pub async fn create(
    session: Session,
    State(db): State<Database>,
    Form(input): Form<CreateFormInput>,
) -> Result<impl IntoResponse, controllers::Error> {
    input.validate()?;
    let id = Paste::insert(
        &db,
        session.user.id,
        input.title,
        input.description,
        input.body,
    )
    .await
    .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;
    Ok(Redirect::to(format!("/pastes/{id}").as_str()).into_response())
}

pub async fn show(
    Path(id): Path<Uuid>,
    session: Option<Session>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::Error> {
    match Paste::find(&db, id)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?
    {
        Some(paste) => Ok((StatusCode::OK, ShowPastesTemplate { session, paste })),
        None => Err(controllers::Error::NotFound),
    }
}

pub async fn destroy(
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::Error> {
    Paste::delete(&db, id)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/pastes"));
    Ok(headers)
}
