use crate::{
    views::pastes::{IndexPastesTemplate, NewPastesTemplate, ShowPastesTemplate},
    Database, Paste,
};
use axum::{
    extract::{Form, Path, State},
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;

pub async fn index(State(db): State<Database>) -> IndexPastesTemplate {
    let conn = db.conn.lock().unwrap();
    let mut statement = conn.prepare("SELECT id, text FROM pastes;").unwrap();
    let mut rows = statement.query(()).unwrap();
    let mut pastes = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        pastes.push(Paste {
            id: row.get(0).unwrap(),
            text: row.get(1).unwrap(),
        })
    }
    IndexPastesTemplate { pastes }
}

pub async fn new() -> NewPastesTemplate {
    NewPastesTemplate {}
}

#[derive(Deserialize, Debug)]
pub struct CreateFormInput {
    pub text: String,
}
pub async fn create(State(db): State<Database>, Form(input): Form<CreateFormInput>) -> Response {
    db.conn
        .lock()
        .unwrap()
        .execute("INSERT INTO pastes (text) VALUES (?1)", params![input.text])
        .unwrap();

    Redirect::to("/pastes").into_response()
}

pub async fn show(Path(id): Path<i64>, State(db): State<Database>) -> impl IntoResponse {
    let maybe_paste = db
        .conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT id, text FROM pastes WHERE id = ?1",
            params![id.to_string()],
            |row| {
                Ok(Paste {
                    id: row.get(0).unwrap(),
                    text: row.get(1).unwrap(),
                })
            },
        )
        .optional()
        .unwrap();

    let status_code = if maybe_paste.is_some() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    };

    (status_code, ShowPastesTemplate { maybe_paste })
}

pub async fn destroy(Path(id): Path<i64>, State(db): State<Database>) -> impl IntoResponse {
    db.conn
        .lock()
        .unwrap()
        .execute("DELETE FROM pastes WHERE id = ?1", params![id])
        .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", "/pastes".parse().unwrap());
    headers
}
