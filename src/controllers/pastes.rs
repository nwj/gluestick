use crate::{
    views::pastes::{NewPasteTemplate, PastesIndexTemplate},
    Db, Paste,
};
use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use ulid::Ulid;

pub async fn index(State(db): State<Db>) -> PastesIndexTemplate {
    let pastes = db.read().unwrap().values().cloned().collect::<Vec<_>>();
    PastesIndexTemplate { pastes }
}

pub async fn new() -> NewPasteTemplate {
    NewPasteTemplate {}
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
