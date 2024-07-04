use actix_web::{
    http::{header::ContentType, StatusCode},
    test, App,
};
use error::AppError;
use models::movie::{Movie, MovieResponse};
use mongodb::bson::oid::ObjectId;
use serde_json::Value;
use services::movie_repo::{MockMovieRepository, MovieRepository};

use super::*;

// General Endpoints

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

// Movie Repo

#[actix_web::test]
async fn test_find_all_movies_ok() {
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

#[actix_web::test]
async fn test_find_all_movies_empty_list() {
    let mut mock = MockMovieRepository::new();

    mock.expect_find_all_movies()
        .returning(|_, _, _| Err(AppError::Empty));

    let result = mock.find_all_movies(None, Some(1), Some(10)).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AppError::Empty);
}

#[actix_web::test]
async fn test_find_movie_by_id_ok() {
    let mut mock = MockMovieRepository::new();
    let oid = ObjectId::new();

    mock.expect_find_movie_by_id().returning(move |_| {
        Ok(Movie {
            _id: oid,
            imdb_id: "tt12345".to_string(),
            title: "El lobo de Wall Street".to_string(),
            director: "Martin Scorsese".to_string(),
            overview: "Testing movies...".to_string(),
            release_date: "2002-12-4".to_string(),
            duration: "2h 54m".to_string(),
            trailer_link: "https://youtube.com/dasDsdXsDS".to_string(),
            genres: vec![
                "Crimen".to_string(),
                "Drama".to_string(),
                "Ciencia Ficción".to_string(),
            ],
            poster: "https://moviedb.com/lobo/lobo_poster.jpg".to_string(),
            backdrop: "https://moviedb.com/lobo/lobo_backdrop.jpg".to_string(),
            review_ids: vec![ObjectId::new()],
        })
    });

    let result = mock.find_movie_by_id(oid.to_string().as_str()).await;
    assert!(result.is_ok());

    let movie = result.unwrap();
    assert_eq!(movie.imdb_id, "tt12345".to_string());
    assert_eq!(movie.title, "El lobo de Wall Street".to_string());
}

#[actix_web::test]
async fn test_find_movie_by_id_not_found() {
    let mut mock = MockMovieRepository::new();
    let oid = ObjectId::new();

    mock.expect_find_movie_by_id()
        .returning(|_| Err(AppError::NotFound));

    let result = mock.find_movie_by_id(oid.to_string().as_str()).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AppError::NotFound);
}

#[actix_web::test]
async fn test_find_movie_by_id_internal_server_error() {
    let mut mock = MockMovieRepository::new();
    let oid = ObjectId::new();

    mock.expect_find_movie_by_id()
        .returning(|_| Err(AppError::InternalServerError));

    let result = mock.find_movie_by_id(oid.to_string().as_str()).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AppError::InternalServerError);
}

#[actix_web::test]
async fn test_find_movie_by_imdb_id_ok() {
    let mut mock = MockMovieRepository::new();
    let imbd_mock_id = "tt12345";

    mock.expect_find_movie_by_imdb_id().returning(move |_| {
        Ok(Movie {
            _id: ObjectId::new(),
            imdb_id: imbd_mock_id.to_string(),
            title: "El lobo de Wall Street".to_string(),
            director: "Martin Scorsese".to_string(),
            overview: "Testing movies...".to_string(),
            release_date: "2002-12-4".to_string(),
            duration: "2h 54m".to_string(),
            trailer_link: "https://youtube.com/dasDsdXsDS".to_string(),
            genres: vec![
                "Crimen".to_string(),
                "Drama".to_string(),
                "Ciencia Ficción".to_string(),
            ],
            poster: "https://moviedb.com/lobo/lobo_poster.jpg".to_string(),
            backdrop: "https://moviedb.com/lobo/lobo_backdrop.jpg".to_string(),
            review_ids: vec![ObjectId::new()],
        })
    });

    let result = mock.find_movie_by_imdb_id(&imbd_mock_id).await;
    assert!(result.is_ok());

    let movie = result.unwrap();
    assert_eq!(movie.imdb_id, "tt12345".to_string());
    assert_eq!(movie.title, "El lobo de Wall Street".to_string());
}

#[actix_web::test]
async fn test_find_movie_by_imdb_id_wrong_imdb_id() {
    let mut mock = MockMovieRepository::new();
    let imdb_mock_id = "tfd2312";

    mock.expect_find_movie_by_imdb_id()
        .returning(|_| Err(AppError::WrongImdbId));

    let result = mock.find_movie_by_imdb_id(&imdb_mock_id).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AppError::WrongImdbId);
}

#[actix_web::test]
async fn test_find_movie_by_imdb_id_not_found() {
    let mut mock = MockMovieRepository::new();
    let imdb_mock_id = "tt54321";

    mock.expect_find_movie_by_imdb_id()
        .returning(|_| Err(AppError::NotFound));

    let result = mock.find_movie_by_imdb_id(&imdb_mock_id).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AppError::NotFound);
}

#[actix_web::test]
async fn test_find_movie_by_imdb_id_internal_server_error() {
    let mut mock = MockMovieRepository::new();
    let imdb_mock_id = "tt54321";

    mock.expect_find_movie_by_imdb_id()
        .returning(|_| Err(AppError::InternalServerError));

    let result = mock.find_movie_by_imdb_id(&imdb_mock_id).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AppError::InternalServerError);
}

#[actix_web::test]
async fn test_create_movie_ok() {
    let mut mock = MockMovieRepository::new();
    let oid = ObjectId::new();
    let movie = Movie {
        _id: oid,
        imdb_id: "tt12345".to_string(),
        title: "El lobo de Wall Street".to_string(),
        director: "Martin Scorsese".to_string(),
        overview: "Testing movies...".to_string(),
        release_date: "2002-12-4".to_string(),
        duration: "2h 54m".to_string(),
        trailer_link: "https://youtube.com/dasDsdXsDS".to_string(),
        genres: vec![
            "Crimen".to_string(),
            "Drama".to_string(),
            "Ciencia Ficción".to_string(),
        ],
        poster: "https://moviedb.com/lobo/lobo_poster.jpg".to_string(),
        backdrop: "https://moviedb.com/lobo/lobo_backdrop.jpg".to_string(),
        review_ids: vec![ObjectId::new()],
    };

    mock.expect_create_movie().returning(move |movie| {
        let mut map_result: Map<String, Value> = Map::new();
        map_result.insert(
            "message".to_string(),
            Value::String(
                format!(
                    "Movie was successfully created. (id: '{}')",
                    movie._id.to_string()
                )
                .to_string(),
            ),
        );
        Ok(map_result)
    });

    let result = mock.create_movie(movie).await;

    assert!(result.is_ok());

    let map = result.unwrap();
    assert_eq!(
        map["message"],
        format!(
            "Movie was successfully created. (id: '{}')",
            oid.to_string()
        )
        .to_string()
    );
}
