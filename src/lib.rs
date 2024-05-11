use crate::db::Database;
use axum::{
    extract::Request,
    routing::{delete, get, post},
    Router,
};
use memory_serve::{load_assets, MemoryServe};
use tower_http::trace::TraceLayer;

pub mod config;
mod controllers;
pub mod db;
mod models;
mod validators;
mod views;

pub fn router(db: Database) -> Router {
    let assets_router = MemoryServe::new(load_assets!("src/assets"))
        .index_file(None)
        .into_router();

    Router::new()
        .route("/", get(controllers::pastes::new))
        .route("/health_check", get(controllers::health_check))
        .route("/signup", get(controllers::users::new))
        .route("/signup", post(controllers::users::create))
        .route("/login", get(controllers::sessions::new))
        .route("/login", post(controllers::sessions::create))
        .route("/pastes", get(controllers::pastes::index))
        .route("/pastes", post(controllers::pastes::create))
        .route("/pastes/:id", get(controllers::pastes::show))
        .route("/pastes/:id", delete(controllers::pastes::destroy))
        .route("/api/pastes", get(controllers::api::pastes::index))
        .route("/api/pastes", post(controllers::api::pastes::create))
        .route("/api/pastes/:id", get(controllers::api::pastes::show))
        .route("/api/pastes/:id", delete(controllers::api::pastes::destroy))
        .fallback(controllers::not_found)
        .nest("/assets", assets_router)
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
