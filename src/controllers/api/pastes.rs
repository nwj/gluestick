use crate::{controllers, db::Database, models::paste::Paste};
use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;

pub async fn index(State(db): State<Database>) -> Result<impl IntoResponse, controllers::Error> {
    let pastes = Paste::all(&db).await?;
    Ok(Json(pastes))
}

#[derive(Debug, Deserialize)]
pub struct CreatePaste {
    title: String,
    description: String,
    body: String,
}

pub async fn create(
    State(db): State<Database>,
    Json(input): Json<CreatePaste>,
) -> Result<impl IntoResponse, controllers::Error> {
    let id = Paste::insert(&db, input.title, input.description, input.body).await?;
    Ok(Json(id))
}
