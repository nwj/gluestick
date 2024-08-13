use crate::controllers::prelude::ErrorTemplate;
use crate::helpers::pagination::CursorPaginationResponse;
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::Username;
use crate::params::pastes::{
    CreatePasteParams, UpdatePasteParams, BODY_REPORT_KEY, DESCRIPTION_REPORT_KEY,
    FILENAME_REPORT_KEY,
};
use crate::params::prelude::Report;
use crate::views::filters;
use askama_axum::Template;
use uuid::Uuid;

#[derive(Debug, Template)]
#[template(path = "pastes/new.html")]
pub struct NewPastesTemplate {
    pub session: Option<Session>,
    pub filename: String,
    pub description: String,
    pub body: String,
    pub visibility: String,
    pub error_report: Report,
}

impl NewPastesTemplate {
    pub fn from_session(session: Session) -> Self {
        Self {
            session: Some(session),
            ..Default::default()
        }
    }

    pub fn from_session_and_params(session: Session, params: CreatePasteParams) -> Self {
        Self {
            session: Some(session),
            filename: params.filename.into(),
            description: params.description.into(),
            body: params.body.into(),
            visibility: params.visibility.into(),
            ..Default::default()
        }
    }
}

impl Default for NewPastesTemplate {
    fn default() -> Self {
        Self {
            session: Option::default(),
            filename: String::default(),
            description: String::default(),
            body: String::default(),
            visibility: "secret".into(),
            error_report: Report::default(),
        }
    }
}

impl ErrorTemplate for NewPastesTemplate {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
    }

    fn with_report(&mut self, report: Report) {
        self.error_report = report;
    }
}

#[derive(Template)]
#[template(path = "pastes/index.html")]
pub struct IndexPastesTemplate {
    pub session: Option<Session>,
    pub paste_username_html_triples: Vec<(Paste, Username, Option<String>)>,
    pub pagination: CursorPaginationResponse,
}

#[derive(Template)]
#[template(path = "pastes/show.html")]
pub struct ShowPastesTemplate {
    pub session: Option<Session>,
    pub paste: Paste,
    pub username: Username,
    pub syntax_highlighted_html: Option<String>,
}

#[derive(Debug, Default, Template)]
#[template(path = "pastes/edit.html")]
pub struct EditPastesTemplate {
    pub session: Option<Session>,
    pub paste_id: Uuid,
    pub filename: String,
    pub description: String,
    pub body: String,
    pub error_report: Report,
}

impl EditPastesTemplate {
    pub fn from_session_and_params(session: Option<Session>, params: UpdatePasteParams) -> Self {
        Self {
            session,
            filename: params.filename.into(),
            description: params.description.into(),
            body: params.body.into(),
            ..Default::default()
        }
    }
}

impl ErrorTemplate for EditPastesTemplate {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
    }

    fn with_report(&mut self, report: Report) {
        self.error_report = report;
    }
}
