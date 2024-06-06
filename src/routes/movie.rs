use actix_web::{
    get, post, put,
    web::{Data, Json, Path, Query},
    HttpResponse,
};
use serde::Deserialize;

use crate::{
    error::AppError,
    models::movie::{Movie, MovieRequest},
    services::db::Database,
};

#[post("/new")]
pub async fn create_movie(db: Data<Database>, request: Json<MovieRequest>) -> HttpResponse {
    match db
        .create_movie(
            Movie::try_from(MovieRequest {
                imdb_id: request.imdb_id.clone(),
                title: request.title.clone(),
                overview: request.overview.clone(),
                duration: request.duration.clone(),
                director: request.director.clone(),
                release_date: request.release_date.clone(),
                trailer_link: request.trailer_link.clone(),
                genres: request.genres.clone(),
                poster: request.poster.clone(),
                backdrop: request.backdrop.clone(),
            })
            .expect("Error converting request to Movie"),
        )
        .await
    {
        Ok(movie) => HttpResponse::Ok().json(movie),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct Params {
    title: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
}

#[get("/findAll")]
pub async fn get_movies(
    db: Data<Database>,
    params: Query<Params>,
) -> Result<HttpResponse, AppError> {
    match db
        .find_all_movies(
            params.title.clone(),
            params.page.clone(),
            params.size.clone(),
        )
        .await
    {
        Ok(movies) => Ok(HttpResponse::Ok().json(movies)),
        Err(err) => Err(err),
    }
}

#[get("/findById/{id}")]
pub async fn get_movie_by_id(
    db: Data<Database>,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    match db.find_movie_by_id(id.as_str()).await {
        Ok(movie) => Ok(HttpResponse::Ok().json(movie)),
        Err(err) => Err(err),
    }
}

#[get("/findByImdbId/{id}")]
pub async fn get_movie_by_imdb_id(
    db: Data<Database>,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let imdb_id = path.into_inner();
    match db.find_movie_by_imdb_id(imdb_id.as_str()).await {
        Ok(movie) => Ok(HttpResponse::Ok().json(movie)),
        Err(err) => Err(err),
    }
}

// TODO: terminar esto
// #[put("/api/v1/movies/update/{id}")]
// pub async fn update_movie(db: Data<Database>, path: Path<(String,)>) -> HttpResponse {
//     let id = path.into_inner().0;

//     match db.update_movie(id.as_str()).await {
//         Ok(movies) => HttpResponse::Ok().json(movies),
//         Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
//     }
// }
