use axum::http::StatusCode;

pub async fn check() -> StatusCode {
    StatusCode::OK
}
