use axum::{
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

pub async fn health() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))
        .body(Body::from("ok"))
        .expect("health response should be valid")
}
