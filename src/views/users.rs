use askama_axum::Template;

#[derive(Template)]
#[template(path = "users_new.html")]
pub struct NewUsersTemplate {}
