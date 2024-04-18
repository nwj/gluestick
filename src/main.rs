mod controllers;
mod views;

use axum::{
    routing::{get, post},
    Router,
};
use controllers::pastes;
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ulid::Ulid;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = Db::default();

    let app = Router::new()
        .nest_service("/assets", ServeDir::new("src/assets"))
        .route("/", get(pastes::new))
        .route("/paste", post(pastes::create))
        .route("/pastes", get(pastes::index))
        .layer(TraceLayer::new_for_http())
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

type Db = Arc<RwLock<HashMap<Ulid, Paste>>>;

#[derive(Debug, Serialize, Clone)]
struct Paste {
    id: Ulid,
    text: String,
}
