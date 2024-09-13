use crate::controllers::users_controller::ChangePasswordParams;
use crate::helpers::view_helper::filters;
use crate::models::api_session::ApiKey;
use crate::models::session::Session;
use askama_axum::Template;
use secrecy::{ExposeSecret, Secret};

#[derive(Default, Template)]
#[template(path = "users/settings.html")]
pub struct SettingsPage {
    pub session: Option<Session>,
    pub api_keys: Vec<ApiKey>,
    pub change_password_form: ChangePasswordFormPartial,
}

// TODO: replace this partial with a block fragment once askama 0.13.0 releases
#[derive(Clone, Debug, Template)]
#[template(path = "users/partials/change_password_form.html")]
pub struct ChangePasswordFormPartial {
    pub old_password: Secret<String>,
    pub new_password: Secret<String>,
    pub new_password_confirm: Secret<String>,
    pub old_password_error_message: Option<String>,
    pub new_password_error_message: Option<String>,
    pub show_success_message: bool,
}

impl Default for ChangePasswordFormPartial {
    fn default() -> Self {
        Self {
            old_password: Secret::new(String::default()),
            old_password_error_message: Option::default(),
            new_password: Secret::new(String::default()),
            new_password_error_message: Option::default(),
            new_password_confirm: Secret::new(String::default()),
            show_success_message: bool::default(),
        }
    }
}

impl From<ChangePasswordParams> for ChangePasswordFormPartial {
    fn from(params: ChangePasswordParams) -> Self {
        Self {
            old_password: params.old_password,
            new_password: params.new_password,
            new_password_confirm: params.new_password_confirm,
            ..Default::default()
        }
    }
}
