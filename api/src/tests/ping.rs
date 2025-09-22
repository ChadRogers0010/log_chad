#[tokio::test]
async fn ping_returns_expected_json() {
    use crate::{AppState, InMemoryStore, app_builder, config::Config};
    use axum::{body::Body, http};
    use http::{Request, StatusCode};
    use serde_json::Value;
    use std::sync::Arc;
    use std::usize;
    use tower::ServiceExt;

    let db = InMemoryStore::default();
    let state = AppState {
        db: Arc::new(db),
        cfg: Config::_default(),
    };
    let app = app_builder(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/ping")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json["message"], "Ping response from server!");
    assert!(json["id"].is_string());
    assert!(json["timestamp"].is_string());
}
