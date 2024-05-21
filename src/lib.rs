use crate::db::Database;
use axum::{
    extract::Request,
    routing::{delete, get, patch, post, put},
    Router,
};
use memory_serve::{load_assets, MemoryServe};
use tower_http::trace::TraceLayer;

pub mod config;
pub mod controllers;
pub mod db;
pub mod extractors;
pub mod models;
pub mod validators;
pub mod views;

pub fn router(db: Database) -> Router {
    let assets_router = MemoryServe::new(load_assets!("src/assets"))
        .index_file(None)
        .into_router();

    let json_api_router = Router::new()
        .route("/pastes", get(controllers::api::pastes::index))
        .route("/pastes", post(controllers::api::pastes::create))
        .route("/pastes/:id", get(controllers::api::pastes::show))
        .route("/pastes/:id", patch(controllers::api::pastes::update))
        .route("/pastes/:id", delete(controllers::api::pastes::destroy))
        .fallback(controllers::api::not_found);

    Router::new()
        .route("/", get(controllers::index))
        .route("/health", get(controllers::health::check))
        .route("/signup", get(controllers::users::new))
        .route("/signup", post(controllers::users::create))
        .route("/settings", get(controllers::users::show))
        .route("/login", get(controllers::sessions::new))
        .route("/login", post(controllers::sessions::create))
        .route("/logout", delete(controllers::sessions::delete))
        .route("/api_sessions", post(controllers::api_sessions::create))
        .route("/pastes", get(controllers::pastes::index))
        .route("/pastes", post(controllers::pastes::create))
        .route("/pastes/new", get(controllers::pastes::new))
        .route("/pastes/:id", get(controllers::pastes::show))
        .route("/pastes/:id", put(controllers::pastes::update))
        .route("/pastes/:id/edit", get(controllers::pastes::edit))
        .route("/pastes/:id", delete(controllers::pastes::destroy))
        .fallback(controllers::not_found)
        .nest("/api", json_api_router)
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
