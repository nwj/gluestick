mod controllers;
mod views;

use axum::{routing::get, Router};
use controllers::index::index;
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

    let app = Router::new()
        .route("/", get(index))
        .route("/ping", get(|| async { "Pong" }))
        .nest_service("/assets", ServeDir::new("src/assets"))
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[allow(dead_code)]
type Db = Arc<RwLock<HashMap<Ulid, Paste>>>;

#[derive(Debug, Serialize, Clone)]
struct Paste {
    id: Ulid,
    text: String,
}
