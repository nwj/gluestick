use crate::db::Database;
use axum::{
    extract::Request,
    routing::{delete, get, post},
    Router,
};
use std::path::PathBuf;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub mod config;
mod controllers;
pub mod db;
mod models;
mod views;

pub fn router(db: Database) -> Router {
    Router::new()
        .route("/", get(controllers::pastes::new))
        .route("/health_check", get(controllers::health_check))
        .route("/pastes", get(controllers::pastes::index))
        .route("/pastes", post(controllers::pastes::create))
        .route("/pastes/:id", get(controllers::pastes::show))
        .route("/pastes/:id", delete(controllers::pastes::destroy))
        .fallback(controllers::not_found)
        .nest_service(
            "/assets",
            ServeDir::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/assets")),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = uuid::Uuid::now_v7();
                    tracing::info_span!(
                        "request",
                        method = tracing::field::display(request.method()),
                        uri = tracing::field::display(request.uri()),
                        version = tracing::field::debug(request.version()),
                        request_id = tracing::field::display(request_id),
                    )
                })
                // disable failure tracing here since we'll log errors via controllers::Error
                .on_failure(()),
        )
        .with_state(db)
}
