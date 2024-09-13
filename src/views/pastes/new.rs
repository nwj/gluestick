use crate::controllers::pastes_controller::CreateParams;
use crate::models::session::Session;
use crate::models::user::Username;
use askama_axum::Template;

#[derive(Debug, Default, Template)]
#[template(path = "pastes/new.html")]
pub struct NewPage {
    pub session: Option<Session>,
    pub new_pastes_form: NewFormPartial,
}

impl From<Session> for NewPage {
    fn from(value: Session) -> Self {
        Self {
            session: Some(value),
            ..Default::default()
        }
    }
}

// TODO: replace this partial with a block fragment once askama 0.13.0 releases
#[derive(Debug, Template)]
#[template(path = "pastes/partials/new_pastes_form.html")]
pub struct NewFormPartial {
    pub filename: String,
    pub filename_error_message: Option<String>,
    pub description: String,
    pub description_error_message: Option<String>,
    pub body: String,
    pub body_error_message: Option<String>,
    pub visibility: String,
    pub visibility_error_message: Option<String>,
}

impl Default for NewFormPartial {
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

impl From<(Username, CreateParams)> for NewFormPartial {
    fn from(value: (Username, CreateParams)) -> Self {
        Self {
            filename: value.1.filename,
            description: value.1.description,
            body: value.1.body,
            visibility: value.1.visibility,
            ..Default::default()
        }
    }
}
