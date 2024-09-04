use crate::controllers::pastes::CreatePasteParams;
use crate::controllers::pastes::UpdatePasteParams;
use crate::helpers::pagination::CursorPaginationResponse;
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::Username;
use crate::views::filters;
use askama_axum::Template;
use uuid::Uuid;

#[derive(Debug, Default, Template)]
#[template(path = "pastes/new.html")]
pub struct NewPastesTemplate {
    pub session: Option<Session>,
    pub new_pastes_form: NewPastesFormPartial,
}

impl From<Session> for NewPastesTemplate {
    fn from(value: Session) -> Self {
        Self {
            session: Some(value),
            ..Default::default()
        }
    }
}

#[derive(Debug, Template)]
#[template(path = "pastes/partials/new_pastes_form.html")]
pub struct NewPastesFormPartial {
    pub filename: String,
    pub filename_error_message: Option<String>,
    pub description: String,
    pub description_error_message: Option<String>,
    pub body: String,
    pub body_error_message: Option<String>,
    pub visibility: String,
    pub visibility_error_message: Option<String>,
}

impl Default for NewPastesFormPartial {
    fn default() -> Self {
        Self {
            filename: String::default(),
            filename_error_message: Option::default(),
            description: String::default(),
            description_error_message: Option::default(),
            body: String::default(),
            body_error_message: Option::default(),
            visibility: "secret".into(),
            visibility_error_message: Option::default(),
        }
    }
}

impl From<(Username, CreatePasteParams)> for NewPastesFormPartial {
    fn from(value: (Username, CreatePasteParams)) -> Self {
        Self {
            filename: value.1.filename,
            description: value.1.description,
            body: value.1.body,
            visibility: value.1.visibility,
            ..Default::default()
        }
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
    pub edit_pastes_form: EditPastesFormPartial,
}

impl From<(Session, Paste)> for EditPastesTemplate {
    fn from(value: (Session, Paste)) -> Self {
        let (session, paste) = value;
        let username = session.user.username.clone();
        Self {
            session: Some(session),
            paste_id: paste.id,
            filename: paste.filename.to_string(),
            edit_pastes_form: EditPastesFormPartial::from((username, paste)),
        }
    }
}

#[derive(Debug, Default, Template)]
#[template(path = "pastes/partials/edit_pastes_form.html")]
pub struct EditPastesFormPartial {
    pub username: String,
    pub paste_id: Uuid,
    pub filename: String,
    pub filename_error_message: Option<String>,
    pub description: String,
    pub description_error_message: Option<String>,
    pub body: String,
    pub body_error_message: Option<String>,
    pub visibility: String,
    pub visibility_error_message: Option<String>,
}

impl From<(Username, Paste)> for EditPastesFormPartial {
    fn from(value: (Username, Paste)) -> Self {
        let (username, paste) = value;
        Self {
            username: username.to_string(),
            paste_id: paste.id,
            filename: paste.filename.to_string(),
            description: paste.description.to_string(),
            body: paste.body.to_string(),
            visibility: paste.visibility.to_string(),
            ..Default::default()
        }
    }
}

impl From<(Username, Uuid, UpdatePasteParams)> for EditPastesFormPartial {
    fn from(value: (Username, Uuid, UpdatePasteParams)) -> Self {
        let (username, paste_id, params) = value;
        Self {
            username: username.to_string(),
            paste_id,
            filename: params.filename,
            description: params.description,
            body: params.body,
            ..Default::default()
        }
    }
}
