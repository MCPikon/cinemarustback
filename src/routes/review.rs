use std::collections::HashMap;

use actix_web::{
    delete, get, patch, post, put,
    web::{Data, Json, Path, Query},
    HttpResponse,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    error::AppError,
    models::review::{Review, ReviewRequest, ReviewUpdate},
    services::db::Database,
};

#[derive(Debug, Deserialize, IntoParams)]
pub struct Params {
    page: Option<u32>,
    size: Option<u32>,
}

/// Find all reviews
#[utoipa::path(
    path = "/api/v1/reviews/findAll",
    responses(
        (status = 200, description = "List all reviews with pagination", body = [ReviewResponseDoc]),
        (status = 204, description = "Empty List", body = AppError, example = json!(AppError::Empty.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string())),
    ),
    params(
        Params
    ),
    tag = "Reviews"
)]
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

/// Find all reviews by imdbId
#[utoipa::path(
    path = "/api/v1/reviews/findAllByImdbId/{imdbId}",
    responses(
        (status = 200, description = "List all reviews by imdbId with pagination", body = [ReviewResponseDoc]),
        (status = 204, description = "Empty List", body = AppError, example = json!(AppError::Empty.to_string())),
        (status = 400, description = "Wrong ImdbId passed", body = AppError, example = json!(AppError::WrongImdbId.to_string())),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string())),
    ),
    params(
        ("imdbId", description = "Unique imdbId of Movie or Series")
    ),
    tag = "Reviews"
)]
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

/// Find review by id
#[utoipa::path(
    path = "/api/v1/reviews/findById/{id}",
    responses(
        (status = 200, description = "Fetch Review by id", body = ReviewResponseDoc),
        (status = 400, description = "Cannot parse ObjectId", body = AppError, example = json!(AppError::CannotParseObjId.to_string())),
        (status = 404, description = "Not Found", body = AppError, example = json!(AppError::NotFound.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Review")
    ),
    tag = "Reviews"
)]
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

/// Create new review
#[utoipa::path(
    path = "/api/v1/reviews/new",
    responses(
        (status = 201, description = "Created", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Review was successfully created. (id: '1234')".to_string())]))),
        (status = 400, description = "ValidationError", body = AppError, example = json!(AppError::ValidationAppError("title: The review title cannot be empty".to_string()).to_string())),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    request_body = ReviewRequest,
    tag = "Reviews"
)]
#[post("/new")]
pub async fn create_review(
    db: Data<Database>,
    request: Json<ReviewRequest>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;
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

/// Delete review by id
#[utoipa::path(
    path = "/api/v1/reviews/delete/{id}",
    responses(
        (status = 200, description = "Deleted", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Review with id: '1234' was successfully deleted".to_string())]))),
        (status = 400, description = "Cannot parse ObjectId", body = AppError, example = json!(AppError::CannotParseObjId.to_string())),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Review")
    ),
    tag = "Reviews"
)]
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

/// Update review by id
#[utoipa::path(
    path = "/api/v1/reviews/update/{id}",
    responses(
        (status = 200, description = "Updated", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Review with id: '1234' was successfully updated".to_string())]))),
        (status = 400, description = "Cannot parse ObjectId or Validation Error", body = AppError, examples(
            ("Cannot parse ObjectId" = (value = json!(AppError::CannotParseObjId.to_string()))),
            ("ValidationError" = (value = json!(AppError::ValidationAppError("title: The review title cannot be empty".to_string()).to_string())))
        )),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Review")
    ),
    request_body = ReviewRequest,
    tag = "Reviews"
)]
#[put("/update/{id}")]
pub async fn update_review_by_id(
    db: Data<Database>,
    path: Path<String>,
    review: Json<ReviewUpdate>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    review.validate()?;
    match db.update_review(id.as_str(), review.0).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchParams {
    field: String,
    value: String,
}

/// Patch review by id
#[utoipa::path(
    path = "/api/v1/reviews/patch/{id}",
    responses(
        (status = 200, description = "Patched", body = String, content_type = "application/json", example = json!(HashMap::from([("message".to_string(), "Review rating with id: '1234' was successfully patched".to_string())]))),
        (status = 404, description = "Not Exists", body = AppError, example = json!(AppError::NotExists.to_string())),
        (status = 400, description = "Cannot parse ObjectId or Field not allowed", body = AppError, examples(
            ("Cannot parse ObjectId" = (value = json!(AppError::CannotParseObjId.to_string()))),
            ("Field not allowed" = (value = json!(AppError::FieldNotAllowed.to_string())))
        )),
        (status = 500, description = "Internal Server Error", body = AppError, example = json!(AppError::InternalServerError.to_string()))
    ),
    params(
        ("id", description = "Unique ObjectId of Review")
    ),
    request_body = PatchParams,
    tag = "Reviews"
)]
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
