#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::allow_attributes)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]

use crate::db::Database;
use crate::models::session::SessionToken;
use axum::{
    extract::Request,
    routing::{delete, get, patch, post, put},
    Router,
};
use memory_serve::{load_assets, MemoryServe};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer, trace::TraceLayer};

pub mod config;
pub mod controllers;
pub mod db;
pub mod extractors;
pub mod helpers;
pub mod models;
pub mod views;

pub fn router(db: Database) -> Router {
    let assets_router = MemoryServe::new(load_assets!("src/assets"))
        .index_file(None)
        .into_router();

    let json_api_router = Router::new()
        .route("/pastes", get(controllers::api::pastes::index))
        .route("/pastes", post(controllers::api::pastes::create))
        .route("/pastes/:id", get(controllers::api::pastes::show))
        .route("/pastes/:id/raw", get(controllers::api::pastes::show_raw))
        .route("/pastes/:id", patch(controllers::api::pastes::update))
        .route("/pastes/:id", delete(controllers::api::pastes::destroy))
        .fallback(controllers::api::not_found);

    Router::new()
        .nest("/api/v1", json_api_router)
        .nest("/assets", assets_router)
        .route("/", get(controllers::about))
        .route("/health", get(controllers::health::check))
        .route("/signup", get(controllers::users::new))
        .route("/signup", post(controllers::users::create))
        .route(
            "/signup/validate/username",
            post(controllers::users::validate_username),
        )
        .route(
            "/signup/validate/email",
            post(controllers::users::validate_email),
        )
        .route(
            "/signup/validate/password",
            post(controllers::users::validate_password),
        )
        .route("/login", get(controllers::sessions::new))
        .route("/login", post(controllers::sessions::create))
        .route("/logout", delete(controllers::sessions::delete))
        .route("/settings", get(controllers::users::settings))
        .route(
            "/settings/change_password",
            post(controllers::users::change_password),
        )
        .route("/api_sessions", post(controllers::api_sessions::create))
        .route(
            "/api_sessions/:api_key_id",
            delete(controllers::api_sessions::destroy),
        )
        .route("/new", get(controllers::pastes::new))
        .route("/pastes", get(controllers::pastes::index))
        .route("/pastes", post(controllers::pastes::create))
        .route("/:username", get(controllers::users::show))
        .route("/:username/:paste_id", get(controllers::pastes::show))
        .route(
            "/:username/:paste_id/raw",
            get(controllers::pastes::show_raw),
        )
        .route(
            "/:username/:paste_id/download",
            get(controllers::pastes::download),
        )
        .route("/:username/:paste_id", put(controllers::pastes::update))
        .route("/:username/:paste_id/edit", get(controllers::pastes::edit))
        .route("/:username/:paste_id", delete(controllers::pastes::destroy))
        .fallback(controllers::not_found)
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
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(CompressionLayer::new())
        .with_state(db)
}

pub fn background_tasks(mut shutdown_rx: mpsc::Receiver<()>, db: Database) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut every_minute = interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::trace!("received graceful shutdown signal. Telling background tasks to shutdown");
                    break;
                },
                _ = every_minute.tick() => {
                    tracing::trace!("starting per minute background tasks");

                    if let Err(e) = SessionToken::expire_idle(&db).await {
                       tracing::error!("error in background task SessionToken::expire_idle: {e}");
                    }
                    if let Err(e) = SessionToken::expire_absolute(&db).await {
                       tracing::error!("error in background task SessionToken::expire_absolute: {e}");
                    }

                    tracing::trace!("finishing per minute background tasks");
                }
            }
        }

        tracing::trace!("shutting down background tasks");
    })
}
