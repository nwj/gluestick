use crate::controllers;
use axum::http::StatusCode;

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn not_found() -> Result<(), controllers::Error> {
    Err(controllers::Error::NotFound)
}
