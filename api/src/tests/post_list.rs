#[tokio::test]
async fn create_and_list_logs() {
    use crate::{AppState, InMemoryStore, app_builder, config::Config};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use common::LogEntry;
    use serde_json::json;
    use std::sync::Arc;
    use tower::ServiceExt; // for `oneshot`
    // Arrange: create in-memory DB and config

    let db = InMemoryStore::default();
    let state = AppState {
        db: Arc::new(db),
        cfg: Config::_default(),
    };
    let app = app_builder(state);

    // Act: POST /logs twice
    let body1 = json!({ "message": "First log entry" });
    let response1 = app
        .clone()
        .oneshot(
            Request::post("/logs")
                .header("content-type", "application/json")
                .body(Body::from(body1.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response1.status(), StatusCode::CREATED);

    let body2 = json!({ "message": "Second log entry" });
    let response2 = app
        .clone()
        .oneshot(
            Request::post("/logs")
                .header("content-type", "application/json")
                .body(Body::from(body2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response2.status(), StatusCode::CREATED);

    // Act: GET /logs
    let response = app
        .oneshot(Request::get("/logs").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response JSON
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let logs: Vec<LogEntry> = serde_json::from_slice(&body_bytes).unwrap();

    // Assert: we have exactly two logs, in order
    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].message, "First log entry");
    assert_eq!(logs[1].message, "Second log entry");
}
