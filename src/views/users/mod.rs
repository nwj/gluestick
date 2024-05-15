use askama_axum::Template;

#[derive(Template)]
#[template(path = "users/new.html")]
pub struct NewUsersTemplate {
    pub current_user: Option<()>,
}
