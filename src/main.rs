mod error;
mod models;
mod routes;
mod services;
#[cfg(test)]
mod tests;

use std::{collections::HashMap, net::Ipv4Addr};

use actix_web::{
    get,
    middleware::Logger,
    web::{self, Data, ServiceConfig},
    App, HttpResponse, HttpServer, Responder,
};
use env_logger::Env;
use log::info;
use routes::{
    movie::{
        create_movie, delete_movie_by_id, get_movie_by_id, get_movie_by_imdb_id, get_movies,
        patch_movie_by_id, update_movie_by_id,
    },
    review::{
        create_review, delete_review_by_id, get_review_by_id, get_reviews, get_reviews_by_imdb_id,
        patch_review_by_id, update_review_by_id,
    },
    series::{
        create_series, delete_series_by_id, get_series, get_series_by_id, get_series_by_imdb_id,
        patch_series_by_id, update_series_by_id,
    },
};
use serde_json::Map;
use services::db::Database;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

/// Ping
#[utoipa::path(
    path = "/api/v1/ping",
    responses(
        (status = 200, description = "Ping", body = String, example = json!("Pong.")),
    ),
    tag = "General"
)]
#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().json("Pong.")
}

/// Health check
#[utoipa::path(
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Health check", body = String, content_type = "application/json", example = json!(HashMap::from([("status".to_string(), "UP".to_string()), ("message".to_string(), "All systems working correctly.".to_string())]))),
    ),
    tag = "General"
)]
#[get("/health")]
async fn health() -> impl Responder {
    let mut response = Map::new();
    response.insert(
        "status".to_string(),
        serde_json::Value::String("UP".to_string()),
    );
    response.insert(
        "message".to_string(),
        serde_json::Value::String("All systems working correctly.".to_string()),
    );
    HttpResponse::MultiStatus().json(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const PORT: u16 = 8080;

    #[derive(OpenApi)]
    #[openapi(
        servers(
            (url = "http://localhost:8080", description = "Server URL in Development environment")
        ),
        paths(
            ping,
            health,
            routes::movie::get_movies,
            routes::movie::get_movie_by_id,
            routes::movie::get_movie_by_imdb_id,
            routes::movie::create_movie,
            routes::movie::delete_movie_by_id,
            routes::movie::update_movie_by_id,
            routes::movie::patch_movie_by_id,
            routes::series::get_series,
            routes::series::get_series_by_id,
            routes::series::get_series_by_imdb_id,
            routes::series::create_series,
            routes::series::delete_series_by_id,
            routes::series::update_series_by_id,
            routes::series::patch_series_by_id,
            routes::review::get_reviews,
            routes::review::get_reviews_by_imdb_id,
            routes::review::get_review_by_id,
            routes::review::create_review,
            routes::review::delete_review_by_id,
            routes::review::update_review_by_id,
            routes::review::patch_review_by_id
        ),
        components(
            schemas(error::AppError, models::movie::MovieDoc, models::movie::MovieRequest, models::movie::MovieResponse, routes::movie::PatchParams, models::series::SeriesDoc, models::series::SeriesRequest, models::series::SeriesResponse, models::series::Season, models::series::Episode, models::review::ReviewResponseDoc, models::review::ReviewRequest, models::review::ReviewUpdate)
        ),
        tags(
            (name = "General", description = "Some endpoints for general purposes."),
            (name = "Movies", description = "Movies management endpoints."),
            (name = "Series", description = "Series management endpoints."),
            (name = "Reviews", description = "Reviews management endpoints.")
        )
    )]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();

    let db = Database::init().await;
    let db_data = Data::new(db);
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    info!("🚀 API is UP and running on port {}!", PORT);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .configure(routes_config)
            .service(
                SwaggerUi::new("/api/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(Redoc::with_url("/api/redoc", openapi.clone()))
            .service(Scalar::with_url("/api/scalar", openapi.clone()))
            .wrap(Logger::default())
    })
    .bind((Ipv4Addr::UNSPECIFIED, PORT))?
    .run()
    .await
}

pub fn routes_config(conf: &mut ServiceConfig) {
    let scope = web::scope("/api/v1")
        .service(ping)
        .service(health)
        .service(
            web::scope("/movies")
                .service(get_movies)
                .service(get_movie_by_id)
                .service(get_movie_by_imdb_id)
                .service(create_movie)
                .service(delete_movie_by_id)
                .service(update_movie_by_id)
                .service(patch_movie_by_id),
        )
        .service(
            web::scope("/series")
                .service(get_series)
                .service(get_series_by_id)
                .service(get_series_by_imdb_id)
                .service(create_series)
                .service(delete_series_by_id)
                .service(update_series_by_id)
                .service(patch_series_by_id),
        )
        .service(
            web::scope("/reviews")
                .service(get_reviews)
                .service(get_review_by_id)
                .service(get_reviews_by_imdb_id)
                .service(create_review)
                .service(delete_review_by_id)
                .service(update_review_by_id)
                .service(patch_review_by_id),
        );
    conf.service(scope);
}
