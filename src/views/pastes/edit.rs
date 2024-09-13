use crate::controllers::pastes_controller::UpdateParams;
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::Username;
use askama_axum::Template;
use uuid::Uuid;

#[derive(Debug, Default, Template)]
#[template(path = "pastes/edit.html")]
pub struct EditPage {
    pub session: Option<Session>,
    pub paste_id: Uuid,
    pub filename: String,
    pub edit_pastes_form: EditFormPartial,
}

impl From<(Session, Paste)> for EditPage {
    fn from(value: (Session, Paste)) -> Self {
        let (session, paste) = value;
        let username = session.user.username.clone();
        Self {
            session: Some(session),
            paste_id: paste.id,
            filename: paste.filename.to_string(),
            edit_pastes_form: EditFormPartial::from((username, paste)),
        }
    }
}

// TODO: replace this partial with a block fragment once askama 0.13.0 releases
#[derive(Debug, Default, Template)]
#[template(path = "pastes/partials/edit_pastes_form.html")]
pub struct EditFormPartial {
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

impl From<(Username, Paste)> for EditFormPartial {
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

impl From<(Username, Uuid, UpdateParams)> for EditFormPartial {
    fn from(value: (Username, Uuid, UpdateParams)) -> Self {
        let (username, paste_id, params) = value;
        Self {
            username: username.to_string(),
            paste_id,
            filename: params.filename,
            description: params.description,
            body: params.body,
            visibility: params.visibility,
            ..Default::default()
        }
    }
}
