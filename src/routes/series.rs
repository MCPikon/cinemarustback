use std::collections::HashMap;

use actix_web::{
    delete, get, patch, post, put,
    web::{Data, Json, Path, Query},
    HttpResponse,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::{
    error::AppError,
    models::series::{Series, SeriesRequest},
    services::db::Database,
};

#[derive(Debug, Deserialize, IntoParams)]
pub struct Params {
    title: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
}

/// Find all series
#[utoipa::path(
    path = "/api/v1/series/findAll",
    responses(
        (status = 200, description = "List all series with pagination", body = [SeriesResponse]),
        (status = 204, description = "Empty List", body = AppError, example = json!(AppError::Empty.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string())),
    ),
    params(
        Params
    ),
    tag = "Series"
)]
#[get("/findAll")]
pub async fn get_series(
    db: Data<Database>,
    params: Query<Params>,
) -> Result<HttpResponse, AppError> {
    match db
        .find_all_series(
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

/// Find series by id
#[utoipa::path(
    path = "/api/v1/series/findById/{id}",
    responses(
        (status = 200, description = "Fetch Series by id", body = SeriesDoc),
        (status = 404, description = "Not Found", body = AppError, example = json!(AppError::NotFound.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Series")
    ),
    tag = "Series"
)]
#[get("/findById/{id}")]
pub async fn get_series_by_id(
    db: Data<Database>,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    match db.find_series_by_id(id.as_str()).await {
        Ok(series) => Ok(HttpResponse::Ok().json(series)),
        Err(err) => Err(err),
    }
}

/// Find series by imdbId
#[utoipa::path(
    path = "/api/v1/series/findByImdbId/{imdbId}",
    responses(
        (status = 200, description = "Fetch Series by imdbId", body = SeriesDoc),
        (status = 400, description = "Wrong ImdbId passed", body = AppError, example = json!(AppError::WrongImdbId.to_string())),
        (status = 404, description = "Not Found", body = AppError, example = json!(AppError::NotFound.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("imdbId", description = "Unique ImdbId of Series")
    ),
    tag = "Series"
)]
#[get("/findByImdbId/{imdbId}")]
pub async fn get_series_by_imdb_id(
    db: Data<Database>,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let imdb_id = path.into_inner();
    match db.find_series_by_imdb_id(imdb_id.as_str()).await {
        Ok(series) => Ok(HttpResponse::Ok().json(series)),
        Err(err) => Err(err),
    }
}

/// Create new series
#[utoipa::path(
    path = "/api/v1/series/new",
    responses(
        (status = 201, description = "Created", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Series was successfully created. (id: '1234')".to_string())]))),
        (status = 400, description = "Already Exists or Validation Error", body = AppError, examples(
            ("AlreadyExists" = (value = json!(AppError::AlreadyExists.to_string()))),
            ("ValidationError" = (value = json!(AppError::ValidationAppError("title: El título de la serie no puede estar vacío.".to_string()).to_string())))
        )),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    request_body = SeriesRequest,
    tag = "Series"
)]
#[post("/new")]
pub async fn create_series(
    db: Data<Database>,
    request: Json<SeriesRequest>,
) -> Result<HttpResponse, AppError> {
    match db
        .create_series(
            Series::try_from(SeriesRequest {
                imdb_id: request.imdb_id.clone(),
                title: request.title.clone(),
                overview: request.overview.clone(),
                number_of_seasons: request.number_of_seasons.clone(),
                creator: request.creator.clone(),
                release_date: request.release_date.clone(),
                trailer_link: request.trailer_link.clone(),
                genres: request.genres.clone(),
                season_list: request.season_list.clone(),
                poster: request.poster.clone(),
                backdrop: request.backdrop.clone(),
            })
            .expect("Error converting request to Series"),
        )
        .await
    {
        Ok(series) => Ok(HttpResponse::Created().json(series)),
        Err(err) => Err(err),
    }
}

/// Delete series by id
#[utoipa::path(
    path = "/api/v1/series/delete/{id}",
    responses(
        (status = 200, description = "Deleted", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Series with id: '1234' was successfully deleted".to_string())]))),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Series")
    ),
    tag = "Series"
)]
#[delete("/delete/{id}")]
pub async fn delete_series_by_id(
    db: Data<Database>,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    match db.delete_series(id.as_str()).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

/// Update series by id
#[utoipa::path(
    path = "/api/v1/series/update/{id}",
    responses(
        (status = 200, description = "Updated", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Series with id: '1234' was successfully updated".to_string())]))),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 400, description = "Wrong ImdbId or ImdbId in use", body = AppError, examples(
            ("Wrong ImdbId" = (value = json!(AppError::WrongImdbId.to_string()))),
            ("ImdbId in use" = (value = json!(AppError::ImdbIdInUse.to_string())))
        )),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Series")
    ),
    request_body = SeriesRequest,
    tag = "Series"
)]
#[put("/update/{id}")]
pub async fn update_series_by_id(
    db: Data<Database>,
    path: Path<String>,
    series: Json<SeriesRequest>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    match db.update_series(id.as_str(), series.0).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchParams {
    field: String,
    value: String,
}

/// Patch series by id
#[utoipa::path(
    path = "/api/v1/series/patch/{id}",
    responses(
        (status = 200, description = "Patched", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Series title with id: '1234' was successfully patched".to_string())]))),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 400, description = "Field not allowed, Wrong ImdbId or ImdbId in use", body = AppError, examples(
            ("Field not allowed" = (value = json!(AppError::FieldNotAllowed.to_string()))),
            ("Wrong ImdbId" = (value = json!(AppError::WrongImdbId.to_string()))),
            ("ImdbId in use" = (value = json!(AppError::ImdbIdInUse.to_string())))
        )),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Series")
    ),
    request_body = PatchParams,
    tag = "Series"
)]
#[patch("/patch/{id}")]
pub async fn patch_series_by_id(
    db: Data<Database>,
    path: Path<String>,
    json_patch: Json<PatchParams>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    match db
        .patch_series(
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
