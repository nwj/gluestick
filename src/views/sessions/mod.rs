use askama_axum::Template;

#[derive(Template)]
#[template(path = "sessions/new.html")]
pub struct NewSessionsTemplate {
    pub optional_user: Option<()>,
}
