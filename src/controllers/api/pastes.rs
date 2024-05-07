use crate::{controllers, db::Database, models::paste::Paste};
use axum::{extract::State, response::IntoResponse, Json};

pub async fn index(State(db): State<Database>) -> Result<impl IntoResponse, controllers::Error> {
    let pastes = Paste::all(&db).await?;
    Ok(Json(pastes))
}
