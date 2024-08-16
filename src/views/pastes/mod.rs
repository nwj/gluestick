use crate::controllers::pastes::CreatePasteParams;
use crate::controllers::prelude::{ErrorTemplate, ErrorTemplate2};
use crate::helpers::pagination::CursorPaginationResponse;
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::Username;
use crate::params::pastes::{
    UpdatePasteParams, BODY_REPORT_KEY, DESCRIPTION_REPORT_KEY, FILENAME_REPORT_KEY,
};
use crate::params::prelude::Report;
use crate::views::filters;
use askama_axum::Template;
use uuid::Uuid;

#[derive(Debug, Template)]
#[template(path = "pastes/new.html")]
pub struct NewPastesTemplate {
    pub session: Option<Session>,
    pub new_pastes_form: NewPastesFormPartial,
}

impl From<Session> for NewPastesTemplate {
    fn from(value: Session) -> Self {
        let username = value.user.username.clone();
        Self {
            session: Some(value),
            new_pastes_form: username.into(),
        }
    }
}

#[derive(Debug, Template)]
#[template(path = "pastes/partials/new_pastes_form.html")]
pub struct NewPastesFormPartial {
    pub username: String,
    pub filename: String,
    pub filename_error_message: Option<String>,
    pub description: String,
    pub description_error_message: Option<String>,
    pub body: String,
    pub body_error_message: Option<String>,
    pub visibility: String,
}

impl Default for NewPastesFormPartial {
    fn default() -> Self {
        Self {
            username: String::default(),
            filename: String::default(),
            filename_error_message: Option::default(),
            description: String::default(),
            description_error_message: Option::default(),
            body: String::default(),
            body_error_message: Option::default(),
            visibility: "secret".into(),
        }
    }
}

impl From<Username> for NewPastesFormPartial {
    fn from(value: Username) -> Self {
        Self {
            username: value.to_string(),
            ..Default::default()
        }
    }
}

impl From<(Username, CreatePasteParams)> for NewPastesFormPartial {
    fn from(value: (Username, CreatePasteParams)) -> Self {
        Self {
            username: value.0.to_string(),
            filename: value.1.filename,
            description: value.1.description,
            body: value.1.body,
            visibility: value.1.visibility,
            ..Default::default()
        }
    }
}

impl ErrorTemplate2 for NewPastesFormPartial {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
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
