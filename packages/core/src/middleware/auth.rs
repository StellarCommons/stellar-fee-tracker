use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

const API_KEY_HEADER: &str = "x-api-key";

pub async fn require_api_key(
    State(expected_key): State<Option<String>>,
    request: Request,
    next: Next,
) -> Response {
    let Some(expected_key) = expected_key else {
        return next.run(request).await;
    };

    let Some(provided_header) = request.headers().get(API_KEY_HEADER) else {
        return unauthorized_response();
    };

    let Ok(provided_key) = provided_header.to_str() else {
        return forbidden_response("Forbidden: invalid API key format");
    };

    if provided_key.is_empty() {
        return forbidden_response("Forbidden: invalid API key format");
    }

    if !constant_time_eq(provided_key.as_bytes(), expected_key.as_bytes()) {
        return unauthorized_response();
    }

    next.run(request).await
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "Unauthorized: missing or invalid API key"
        })),
    )
        .into_response()
}

fn forbidden_response(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "error": message
        })),
    )
        .into_response()
}

// Constant-time byte comparison to avoid timing leaks on key checks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut diff = a.len() ^ b.len();

    for i in 0..max_len {
        let av = *a.get(i).unwrap_or(&0);
        let bv = *b.get(i).unwrap_or(&0);
        diff |= usize::from(av ^ bv);
    }

    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{HeaderValue, header},
        middleware::from_fn_with_state,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn build_test_app(api_key: Option<String>) -> Router {
        let protected = Router::new().route("/protected", get(ok_handler));
        let protected = if api_key.is_some() {
            protected.layer(from_fn_with_state(api_key, require_api_key))
        } else {
            protected
        };

        Router::new()
            .route("/health", get(ok_handler))
            .merge(protected)
    }

    #[tokio::test]
    async fn no_configured_api_key_allows_request() {
        let app = build_test_app(None);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn configured_key_with_correct_header_allows_request() {
        let app = build_test_app(Some("secret".to_string()));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(API_KEY_HEADER, "secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn configured_key_with_missing_header_returns_401() {
        let app = build_test_app(Some("secret".to_string()));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            payload["error"],
            "Unauthorized: missing or invalid API key"
        );
    }

    #[tokio::test]
    async fn configured_key_with_wrong_header_returns_401() {
        let app = build_test_app(Some("secret".to_string()));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(API_KEY_HEADER, "wrong")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            payload["error"],
            "Unauthorized: missing or invalid API key"
        );
    }

    #[tokio::test]
    async fn invalid_header_format_returns_403() {
        let app = build_test_app(Some("secret".to_string()));
        let mut request = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();
        request.headers_mut().insert(
            header::HeaderName::from_static(API_KEY_HEADER),
            HeaderValue::from_bytes(&[0xff, 0xfe]).unwrap(),
        );

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["error"], "Forbidden: invalid API key format");
    }

    #[tokio::test]
    async fn health_route_stays_public_even_when_key_is_configured() {
        let app = build_test_app(Some("secret".to_string()));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn constant_time_eq_checks_full_input() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }
}
