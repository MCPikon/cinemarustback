use std::collections::HashMap;

use crate::{
    error::AppError,
    models::movie::{Movie, MovieRequest},
    services::db::Database,
};
use actix_web::{
    delete, get, patch, post, put,
    web::{Data, Json, Path, Query},
    HttpResponse,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, IntoParams)]
pub struct Params {
    title: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
}

/// Find all movies
#[utoipa::path(
    path = "/api/v1/movies/findAll",
    responses(
        (status = 200, description = "List all movies with pagination", body = [MovieResponse]),
        (status = 204, description = "Empty List", body = AppError, example = json!(AppError::Empty.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string())),
    ),
    params(
        Params
    ),
    tag = "Movies"
)]
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
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

/// Find movie by id
#[utoipa::path(
    path = "/api/v1/movies/findById/{id}",
    responses(
        (status = 200, description = "Fetch Movie by id", body = MovieDoc),
        (status = 400, description = "Cannot parse ObjectId", body = AppError, example = json!(AppError::CannotParseObjId.to_string())),
        (status = 404, description = "Not Found", body = AppError, example = json!(AppError::NotFound.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Movie")
    ),
    tag = "Movies"
)]
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

/// Find movie by imdbId
#[utoipa::path(
    path = "/api/v1/movies/findByImdbId/{imdbId}",
    responses(
        (status = 200, description = "Fetch Movie by imdbId", body = MovieDoc),
        (status = 400, description = "Wrong ImdbId passed", body = AppError, example = json!(AppError::WrongImdbId.to_string())),
        (status = 404, description = "Not Found", body = AppError, example = json!(AppError::NotFound.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("imdbId", description = "Unique imdbId of Movie")
    ),
    tag = "Movies"
)]
#[get("/findByImdbId/{imdbId}")]
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

/// Create new movie
#[utoipa::path(
    path = "/api/v1/movies/new",
    responses(
        (status = 201, description = "Created", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Movie was successfully created. (id: '1234')".to_string())]))),
        (status = 400, description = "Already Exists or Validation Error", body = AppError, examples(
            ("AlreadyExists" = (value = json!(AppError::AlreadyExists.to_string()))),
            ("ValidationError" = (value = json!(AppError::ValidationAppError("title: The movie title cannot be empty".to_string()).to_string())))
        )),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    request_body = MovieRequest,
    tag = "Movies"
)]
#[post("/new")]
pub async fn create_movie(
    db: Data<Database>,
    request: Json<MovieRequest>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
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
        Ok(movie) => Ok(HttpResponse::Created().json(movie)),
        Err(err) => Err(err),
    }
}

/// Delete movie by id
#[utoipa::path(
    path = "/api/v1/movies/delete/{id}",
    responses(
        (status = 200, description = "Deleted", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Movie with id: '1234' was successfully deleted".to_string())]))),
        (status = 400, description = "Cannot parse ObjectId", body = AppError, example = json!(AppError::CannotParseObjId.to_string())),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Movie")
    ),
    tag = "Movies"
)]
#[delete("/delete/{id}")]
pub async fn delete_movie_by_id(
    db: Data<Database>,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    match db.delete_movie(id.as_str()).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

/// Update movie by id
#[utoipa::path(
    path = "/api/v1/movies/update/{id}",
    responses(
        (status = 200, description = "Updated", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Movie with id: '1234' was successfully updated".to_string())]))),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 400, description = "Validation Error, Cannot parse ObjectId or ImdbId in use", body = AppError, examples(
            ("ValidationError" = (value = json!(AppError::ValidationAppError("title: The movie title cannot be empty".to_string()).to_string()))),
            ("Cannot parse ObjectId" = (value = json!(AppError::CannotParseObjId.to_string()))),
            ("ImdbId in use" = (value = json!(AppError::ImdbIdInUse.to_string())))
        )),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Movie")
    ),
    request_body = MovieRequest,
    tag = "Movies"
)]
#[put("/update/{id}")]
pub async fn update_movie_by_id(
    db: Data<Database>,
    path: Path<String>,
    movie: Json<MovieRequest>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    movie.validate()?;
    match db.update_movie(id.as_str(), movie.0).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchParams {
    field: String,
    value: String,
}

/// Patch movie by id
#[utoipa::path(
    path = "/api/v1/movies/patch/{id}",
    responses(
        (status = 200, description = "Patched", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Movie title with id: '1234' was successfully patched".to_string())]))),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 400, description = "Field not allowed, Cannot parse ObjectId, Wrong ImdbId or ImdbId in use", body = AppError, examples(
            ("Field not allowed" = (value = json!(AppError::FieldNotAllowed.to_string()))),
            ("Cannot parse ObjectId" = (value = json!(AppError::CannotParseObjId.to_string()))),
            ("Wrong ImdbId" = (value = json!(AppError::WrongImdbId.to_string()))),
            ("ImdbId in use" = (value = json!(AppError::ImdbIdInUse.to_string())))
        )),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Movie")
    ),
    request_body = PatchParams,
    tag = "Movies"
)]
#[patch("/patch/{id}")]
pub async fn patch_movie_by_id(
    db: Data<Database>,
    path: Path<String>,
    json_patch: Json<PatchParams>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    match db
        .patch_movie(
            id.as_str(),
            json_patch.0.field.as_str(),
            json_patch.0.value.as_str(),
        )
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}
