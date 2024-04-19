use crate::{
    views::pastes::{IndexPastesTemplate, NewPastesTemplate, ShowPastesTemplate},
    Db, Paste,
};
use axum::{
    extract::{Form, Path, State},
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use ulid::Ulid;

pub async fn index(State(db): State<Db>) -> IndexPastesTemplate {
    let pastes = db.read().unwrap().values().cloned().collect::<Vec<_>>();
    IndexPastesTemplate { pastes }
}

pub async fn new() -> NewPastesTemplate {
    NewPastesTemplate {}
}

#[derive(Deserialize, Debug)]
pub struct CreateFormInput {
    pub text: String,
}
pub async fn create(State(db): State<Db>, Form(input): Form<CreateFormInput>) -> Response {
    let paste = Paste {
        id: Ulid::new(),
        text: input.text,
    };
    db.write().unwrap().insert(paste.id, paste.clone());

    Redirect::to("/pastes").into_response()
}

pub async fn show(Path(id): Path<Ulid>, State(db): State<Db>) -> impl IntoResponse {
    let maybe_paste = db.read().unwrap().get(&id).cloned();

    let status_code = if maybe_paste.is_some() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    };

    (status_code, ShowPastesTemplate { maybe_paste })
}

pub async fn destroy(Path(id): Path<Ulid>, State(db): State<Db>) -> impl IntoResponse {
    db.write().unwrap().remove(&id);

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", "/pastes".parse().unwrap());
    headers
}
