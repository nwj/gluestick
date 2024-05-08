use crate::{
    controllers,
    db::Database,
    models::paste::Paste,
    views::pastes::{IndexPastesTemplate, NewPastesTemplate, ShowPastesTemplate},
};
use axum::{
    extract::{Form, Path, State},
    http::{header::HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use uuid::Uuid;

pub async fn index(State(db): State<Database>) -> Result<impl IntoResponse, controllers::Error> {
    let pastes = Paste::all(&db).await?;
    Ok(IndexPastesTemplate { pastes })
}

pub async fn new() -> NewPastesTemplate {
    NewPastesTemplate {}
}

#[derive(Deserialize, Debug)]
pub struct CreateFormInput {
    pub title: String,
    pub description: String,
    pub body: String,
}

pub async fn create(
    State(db): State<Database>,
    Form(input): Form<CreateFormInput>,
) -> Result<impl IntoResponse, controllers::Error> {
    let id = Paste::insert(&db, input.title, input.description, input.body).await?;
    Ok(Redirect::to(format!("/pastes/{}", id).as_str()).into_response())
}

pub async fn show(
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::Error> {
    match Paste::find(&db, id).await? {
        Some(paste) => Ok((StatusCode::OK, ShowPastesTemplate { paste })),
        None => Err(controllers::Error::NotFound),
    }
}

pub async fn destroy(
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::Error> {
    Paste::delete(&db, id).await?;
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/pastes"));
    Ok(headers)
}
