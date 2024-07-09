use actix_web::{
    http::{header::ContentType, StatusCode},
    test, App,
};

use super::*;

// Integration Tests

#[actix_web::test]
async fn test_ping_ok() {
    let app = test::init_service(App::new().service(ping)).await;
    let req = test::TestRequest::get()
        .uri("/ping")
        .insert_header(ContentType::plaintext())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert_eq!(String::from_utf8_lossy(&body), "\"Pong.\"")
}

#[actix_web::test]
async fn test_health_ok() {
    let app = test::init_service(App::new().service(health)).await;
    let req = test::TestRequest::get()
        .uri("/health")
        .insert_header(ContentType::json())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::MULTI_STATUS);
    let body = test::read_body(resp).await;
    let mut expected_res = Map::new();
    expected_res.insert(
        "status".to_string(),
        serde_json::Value::String("UP".to_string()),
    );
    expected_res.insert(
        "message".to_string(),
        serde_json::Value::String("All systems working correctly.".to_string()),
    );
    assert_eq!(
        String::from_utf8_lossy(&body),
        serde_json::to_string(&expected_res).unwrap()
    )
}
