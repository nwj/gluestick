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
    pub description: String,
    pub body: String,
}

pub async fn create(
    State(db): State<Database>,
    Form(input): Form<CreateFormInput>,
) -> Result<impl IntoResponse, controllers::Error> {
    Paste::insert(&db, input.description, input.body).await?;
    Ok(Redirect::to("/pastes").into_response())
}

pub async fn show(
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::Error> {
    let maybe_paste = Paste::find(&db, id).await?;
    if maybe_paste.is_some() {
        Ok((StatusCode::OK, ShowPastesTemplate { maybe_paste }))
    } else {
        Err(controllers::Error::NotFound)
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
