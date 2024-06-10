mod error;
mod models;
mod routes;
mod services;

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
    series::{
        create_series, delete_series_by_id, get_series, get_series_by_id, get_series_by_imdb_id,
        patch_series_by_id, update_series_by_id,
    },
};
use serde_json::Map;
use services::db::Database;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json("The CinemaRustBack API is running!!")
}

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

    let db = Database::init().await;
    let db_data = Data::new(db);
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    info!("🚀 API is UP and running on port {}!", PORT);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .configure(routes_config)
            .wrap(Logger::default())
    })
    .bind(("localhost", PORT))?
    .run()
    .await
}

pub fn routes_config(conf: &mut ServiceConfig) {
    let scope = web::scope("/api/v1")
        .service(hello)
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
        );
    conf.service(scope);
}
