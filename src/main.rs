mod models;
mod routes;
mod services;

use actix_web::{get, middleware::Logger, web::Data, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use log::info;
use routes::movie::get_movies;
use services::db::Database;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json("The CinemaRustBack API is running!!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Database::init().await;
    let db_data = Data::new(db);
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    info!("API is UP and running on port 8080!");
    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .wrap(Logger::default())
            .service(hello)
            .service(get_movies)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
