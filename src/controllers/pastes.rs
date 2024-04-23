use crate::{
    controllers::AppError,
    db::Database,
    models::paste::Paste,
    views::pastes::{IndexPastesTemplate, NewPastesTemplate, ShowPastesTemplate},
};
use axum::{
    extract::{Form, Path, State},
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;

pub async fn index(State(db): State<Database>) -> Result<impl IntoResponse, AppError> {
    let pastes = Paste::all(&db).await?;
    Ok(IndexPastesTemplate { pastes })
}

pub async fn new() -> NewPastesTemplate {
    NewPastesTemplate {}
}

#[derive(Deserialize, Debug)]
pub struct CreateFormInput {
    pub text: String,
}

pub async fn create(
    State(db): State<Database>,
    Form(input): Form<CreateFormInput>,
) -> Result<impl IntoResponse, AppError> {
    Paste::insert(&db, input.text).await?;
    Ok(Redirect::to("/pastes").into_response())
}

pub async fn show(
    Path(id): Path<i64>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, AppError> {
    let maybe_paste = Paste::find(&db, id).await?;
    if maybe_paste.is_some() {
        Ok((StatusCode::OK, ShowPastesTemplate { maybe_paste }))
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn destroy(
    Path(id): Path<i64>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, AppError> {
    Paste::delete(&db, id).await?;
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", "/pastes".parse().unwrap());
    Ok(headers)
}
