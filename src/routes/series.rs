use actix_web::{
    delete, get, patch, post, put,
    web::{Data, Json, Path, Query},
    HttpResponse,
};
use serde::Deserialize;

use crate::{
    error::AppError,
    models::series::{Series, SeriesRequest},
    services::db::Database,
};

#[derive(Debug, Deserialize)]
pub struct Params {
    title: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
}

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

#[derive(Debug, Deserialize)]
pub struct PatchParams {
    field: String,
    value: String,
}

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
