#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::allow_attributes)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::too_many_lines)]

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
use tracing::Level;

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
        .route("/pastes", get(controllers::api::pastes_controller::index))
        .route("/pastes", post(controllers::api::pastes_controller::create))
        .route(
            "/pastes/:id",
            get(controllers::api::pastes_controller::show),
        )
        .route(
            "/pastes/:id/raw",
            get(controllers::api::pastes_controller::show_raw),
        )
        .route(
            "/pastes/:id",
            patch(controllers::api::pastes_controller::update),
        )
        .route(
            "/pastes/:id",
            delete(controllers::api::pastes_controller::destroy),
        )
        .fallback(controllers::api::application_controller::not_found);

    Router::new()
        .nest("/api/v1", json_api_router)
        .nest("/assets", assets_router)
        .route("/", get(controllers::application_controller::index))
        .route(
            "/health",
            get(controllers::application_controller::health_check),
        )
        .route("/signup", get(controllers::users_controller::new))
        .route("/signup", post(controllers::users_controller::create))
        .route(
            "/signup/validate/username",
            post(controllers::users_controller::validate_username),
        )
        .route(
            "/signup/validate/email",
            post(controllers::users_controller::validate_email),
        )
        .route(
            "/signup/validate/password",
            post(controllers::users_controller::validate_password),
        )
        .route("/login", get(controllers::sessions_controller::new))
        .route("/login", post(controllers::sessions_controller::create))
        .route("/logout", delete(controllers::sessions_controller::delete))
        .route("/settings", get(controllers::users_controller::settings))
        .route(
            "/settings/change_password",
            post(controllers::users_controller::change_password),
        )
        .route(
            "/api_sessions",
            post(controllers::api_sessions_controller::create),
        )
        .route(
            "/api_sessions/:api_key_id",
            delete(controllers::api_sessions_controller::destroy),
        )
        .route("/new", get(controllers::pastes_controller::new))
        .route("/pastes", get(controllers::pastes_controller::index))
        .route("/pastes", post(controllers::pastes_controller::create))
        .route("/:username", get(controllers::users_controller::show))
        .route(
            "/:username/:paste_id",
            get(controllers::pastes_controller::show),
        )
        .route(
            "/:username/:paste_id/raw",
            get(controllers::pastes_controller::show_raw),
        )
        .route(
            "/:username/:paste_id/download",
            get(controllers::pastes_controller::download),
        )
        .route(
            "/:username/:paste_id",
            put(controllers::pastes_controller::update),
        )
        .route(
            "/:username/:paste_id/edit",
            get(controllers::pastes_controller::edit),
        )
        .route(
            "/:username/:paste_id",
            delete(controllers::pastes_controller::destroy),
        )
        .fallback(controllers::application_controller::not_found)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = uuid::Uuid::now_v7();
                    tracing::info_span!(
                        "request",
                        method = tracing::field::display(request.method()),
                        uri = tracing::field::display(request.uri()),
                        request_id = tracing::field::display(request_id),
                        session = tracing::field::Empty,
                        api_session = tracing::field::Empty,
                    )
                })
                .on_request(tower_http::trace::DefaultOnRequest::new().level(Level::TRACE))
                .on_response(tower_http::trace::DefaultOnResponse::new().level(Level::TRACE))
                .on_failure(tower_http::trace::DefaultOnFailure::new().level(Level::ERROR)),
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
                    tracing::info!("received graceful shutdown signal. Telling background tasks to shutdown");
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

        tracing::info!("shutting down background tasks");
    })
}
