use axum::{
    routing::{delete, get, post},
    Router,
};
use std::path::PathBuf;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod controllers;
mod db;
mod models;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // We won't use .env files in production, so only compile this in non-release builds
    #[cfg(debug_assertions)]
    dotenvy::dotenv()
        .map_err(|e| {
            tracing::debug!("did not find a .env file, continuing with normal execution");
            e
        })
        .ok();

    let config = config::Config::parse()?;

    let mut db = db::Database::init().await?;

    // Pragmas should be applied immediately after connecting to the database and outside of
    // the context of migrations, because some (e.g. `foreign keys`) need to be executed per
    // connection and some (e.g. `journal_mode`) need to be executed outside of transactions,
    // which migrations are run in.
    db.conn
        .call(|conn| {
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.pragma_update(None, "busy_timeout", "5000")?;
            conn.pragma_update(None, "foreign_keys", "true")?;
            Ok(())
        })
        .await?;

    db::migrations().to_latest(&mut db.conn).await?;

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

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", config.port())).await?;

    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
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
