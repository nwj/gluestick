use askama_axum::Template;

#[derive(Template)]
#[template(path = "sessions_new.html")]
pub struct NewSessionsTemplate {
    pub optional_user: Option<()>,
}
