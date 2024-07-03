use actix_web::{
    body::to_bytes,
    http::{header::ContentType, StatusCode},
    test, App,
};
use mockall::predicate::*;
use models::movie::MovieResponse;
use services::movie_repo::{MockMovieRepository, MovieRepository};
use web::Bytes;

use super::*;

trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

#[actix_web::test]
async fn test_ping_ok() {
    let app = test::init_service(App::new().service(ping)).await;
    let req = test::TestRequest::get()
        .uri("/ping")
        .insert_header(ContentType::plaintext())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), "\"Pong.\"")
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
    let body = to_bytes(resp.into_body()).await.unwrap();
    let expected_res = HashMap::from([
        (
            "message".to_string(),
            "All systems working correctly.".to_string(),
        ),
        ("status".to_string(), "UP".to_string()),
    ]);
    assert_eq!(body.as_str(), serde_json::to_string(&expected_res).unwrap())
}

#[actix_web::test]
async fn test_get_movies() {
    let mut mock = MockMovieRepository::new();

    mock.expect_find_all_movies().returning(|_, _, _| {
        let mut result_map = serde_json::Map::new();
        let movie = MovieResponse {
            imdb_id: "tt12345".to_string(),
            title: "Casino".to_string(),
            duration: "2h 54m".to_string(),
            release_date: "1990-3-4".to_string(),
            poster: "https://moviedb.com/casino/poster.jpg".to_string(),
        };
        result_map.insert(
            "movies".to_string(),
            serde_json::to_value(vec![movie]).unwrap(),
        );
        result_map.insert("currentPage".to_string(), serde_json::to_value(1).unwrap());
        result_map.insert("totalItems".to_string(), serde_json::to_value(1).unwrap());
        result_map.insert("totalPages".to_string(), serde_json::to_value(1).unwrap());
        Ok(result_map)
    });

    let result = mock
        .find_all_movies(Some("Casino".to_string()), Some(1), Some(10))
        .await;
    assert!(result.is_ok());

    let map = result.unwrap();
    assert_eq!(map.get("currentPage").unwrap(), 1);
    assert_eq!(map.get("totalItems").unwrap(), 1);
    assert_eq!(map.get("totalPages").unwrap(), 1);

    let movie_list = map.get("movies").unwrap().as_array().unwrap();
    assert_eq!(movie_list.len(), 1);
    assert_eq!(movie_list[0].get("title").unwrap(), "Casino");
}
