mod controllers;
mod views;

use axum::{
    routing::{delete, get, post},
    Router,
};
use controllers::pastes;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = Database::new();

    db.conn
        .lock()
        .unwrap()
        .execute(
            "CREATE TABLE pastes (id INTEGER PRIMARY KEY, text TEXT NOT NULL) STRICT;",
            (),
        )
        .unwrap();

    let app = Router::new()
        .route("/", get(pastes::new))
        .route("/pastes", get(pastes::index))
        .route("/pastes", post(pastes::create))
        .route("/pastes/:id", get(pastes::show))
        .route("/pastes/:id", delete(pastes::destroy))
        .layer(TraceLayer::new_for_http())
        .nest_service("/assets", ServeDir::new("src/assets"))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new() -> Self {
        let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        Self { conn }
    }
}

#[derive(Debug)]
struct Paste {
    id: i64,
    text: String,
}
