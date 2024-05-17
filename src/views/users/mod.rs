use askama_axum::Template;

#[derive(Template)]
#[template(path = "users/new.html")]
pub struct NewUsersTemplate {
    pub session: Option<()>,
}
