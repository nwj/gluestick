use axum::{
    routing::{delete, get, post},
    Router,
};
use std::path::PathBuf;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod controllers;
mod db;
mod models;
mod views;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = db::Database::init().await.unwrap();

    let app = Router::new()
        .route("/", get(controllers::pastes::new))
        .route("/pastes", get(controllers::pastes::index))
        .route("/pastes", post(controllers::pastes::create))
        .route("/pastes/:id", get(controllers::pastes::show))
        .route("/pastes/:id", delete(controllers::pastes::destroy))
        .route("/errors/404", get(controllers::not_found))
        .route("/errors/500", get(controllers::internal_server_error))
        .fallback(controllers::not_found)
        .nest_service(
            "/assets",
            ServeDir::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/assets")),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler.");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler.")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
