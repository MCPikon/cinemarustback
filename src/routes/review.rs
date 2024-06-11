use actix_web::{
    get,
    web::{Data, Path, Query},
    HttpResponse,
};
use serde::Deserialize;

use crate::{error::AppError, services::db::Database};

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u32>,
    size: Option<u32>,
}

#[get("/findAll")]
pub async fn get_reviews(
    db: Data<Database>,
    params: Query<Params>,
) -> Result<HttpResponse, AppError> {
    match db
        .find_all_reviews(params.page.clone(), params.size.clone())
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

#[get("/findById/{id}")]
pub async fn get_review_by_id(
    db: Data<Database>,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    match db.find_review_by_id(id.as_str()).await {
        Ok(review) => Ok(HttpResponse::Ok().json(review)),
        Err(err) => Err(err),
    }
}

#[get("/findAllByImdbId/{imdbId}")]
pub async fn get_reviews_by_imdb_id(
    db: Data<Database>,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let imdb_id = path.into_inner();
    match db.find_all_reviews_by_imdb_id(imdb_id.as_str()).await {
        Ok(review_list) => Ok(HttpResponse::Ok().json(review_list)),
        Err(err) => Err(err),
    }
}
