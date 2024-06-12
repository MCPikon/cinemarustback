use actix_web::{
    delete, get, patch, post, put,
    web::{Data, Json, Path, Query},
    HttpResponse,
};
use serde::Deserialize;

use crate::{
    error::AppError,
    models::review::{Review, ReviewRequest, ReviewUpdate},
    services::db::Database,
};

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

#[post("/new")]
pub async fn create_review(
    db: Data<Database>,
    request: Json<ReviewRequest>,
) -> Result<HttpResponse, AppError> {
    match db
        .create_review(
            Review::try_from(ReviewRequest {
                title: request.title.clone(),
                rating: request.rating.clone(),
                body: request.title.clone(),
                imdb_id: request.imdb_id.clone(),
            })
            .expect("Error converting request to Review"),
            request.imdb_id.as_str(),
        )
        .await
    {
        Ok(res) => Ok(HttpResponse::Created().json(res)),
        Err(err) => Err(err),
    }
}

#[delete("/delete/{id}")]
pub async fn delete_review_by_id(
    db: Data<Database>,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    match db.delete_review(id.as_str()).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

#[put("/update/{id}")]
pub async fn update_review_by_id(
    db: Data<Database>,
    path: Path<String>,
    review: Json<ReviewUpdate>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    match db.update_review(id.as_str(), review.0).await {
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
pub async fn patch_review_by_id(
    db: Data<Database>,
    path: Path<String>,
    json_patch: Json<PatchParams>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    match db
        .patch_review(
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
