use std::str::FromStr;

use crate::{
    error::AppError,
    models::review::{Review, ReviewResponse, ReviewUpdate},
};
use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use log::{error, info, warn};
use mongodb::{
    bson::{doc, oid::ObjectId, to_bson, DateTime},
    options::{CountOptions, FindOptions},
};
use serde_json::{Map, Value};

use super::{db::Database, movie_repo::MovieRepository, series_repo::SeriesRepository};

lazy_static! {
    static ref RE_IMDB_ID: regex::Regex = regex::Regex::new(r"^tt\d+$").unwrap();
}

#[async_trait]
pub trait ReviewRepository {
    async fn find_all_reviews(
        &self,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<Map<String, Value>, AppError>;
    async fn find_all_reviews_by_imdb_id(
        &self,
        imdb_id: &str,
    ) -> Result<Vec<ReviewResponse>, AppError>;
    async fn find_review_by_id(&self, id: &str) -> Result<ReviewResponse, AppError>;
    async fn create_review(
        &self,
        review: Review,
        imdb_id: &str,
    ) -> Result<Map<String, Value>, AppError>;
    async fn movie_exists_by_review_id(
        &self,
        review_id: ObjectId,
    ) -> Result<(bool, Option<ObjectId>), AppError>;
    async fn series_exists_by_review_id(
        &self,
        review_id: ObjectId,
    ) -> Result<(bool, Option<ObjectId>), AppError>;
    async fn delete_review(&self, id: &str) -> Result<Map<String, Value>, AppError>;
    async fn update_review(
        &self,
        id: &str,
        review: ReviewUpdate,
    ) -> Result<Map<String, Value>, AppError>;
    async fn patch_review(
        &self,
        id: &str,
        field: &str,
        val: &str,
    ) -> Result<Map<String, Value>, AppError>;
}

#[async_trait]
impl ReviewRepository for Database {
    async fn find_all_reviews(
        &self,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<Map<String, Value>, AppError> {
        info!("GET reviews /findAll executed");
        let mut result_map: Map<String, Value> = Map::new();

        let page_num = match page {
            None => 0,
            Some(page) => {
                if page > 0 {
                    page
                } else {
                    0
                }
            }
        };
        let page_size = match size {
            None => 10,
            Some(size) => {
                if size > 0 {
                    size
                } else {
                    10
                }
            }
        };

        let total_items = self
            .reviews
            .count_documents(None, CountOptions::default())
            .await
            .ok()
            .expect("Error counting total of reviews");
        let total_pages = (total_items as f64 / page_size as f64).ceil() as u64;

        let options = FindOptions::builder()
            .skip((page_num * page_size) as u64)
            .limit(page_size as i64)
            .build();

        let cursor = self
            .reviews
            .find(None, options)
            .await
            .ok()
            .expect("Error finding all reviews");

        let review_list: Vec<ReviewResponse> = cursor
            .map(|review| ReviewResponse::try_from(review.unwrap()))
            .try_collect()
            .await
            .ok()
            .expect("Error collecting reviews");

        if review_list.is_empty() {
            warn!("Warn in reviews /findAll [{}]", AppError::Empty.to_string());
            return Err(AppError::Empty);
        }

        result_map.insert(
            "reviews".to_string(),
            serde_json::to_value(review_list).unwrap(),
        );
        result_map.insert(
            "currentPage".to_string(),
            serde_json::to_value(page_num).unwrap(),
        );
        result_map.insert(
            "totalItems".to_string(),
            serde_json::to_value(total_items).unwrap(),
        );
        result_map.insert(
            "totalPages".to_string(),
            serde_json::to_value(total_pages).unwrap(),
        );

        Ok(result_map)
    }

    async fn find_all_reviews_by_imdb_id(
        &self,
        imdb_id: &str,
    ) -> Result<Vec<ReviewResponse>, AppError> {
        info!(
            "GET reviews /findAllByImdbId with imdbId: '{}' executed",
            imdb_id
        );

        if !RE_IMDB_ID.is_match(imdb_id) {
            error!(
                "Error in reviews /findAllByImdbId with imdbId: '{}' [{}]",
                imdb_id,
                AppError::WrongImdbId.to_string()
            );
            return Err(AppError::WrongImdbId);
        }

        let reviews_id_list: Vec<ObjectId>;
        if self.movie_exists_by_imdb_id(imdb_id).await? {
            reviews_id_list = match self.movies.find_one(doc! {"imdbId": imdb_id}, None).await {
                Ok(movie) => movie.unwrap().review_ids,
                Err(_) => {
                    error!(
                        "Error in reviews /findAllByImdbId with imdbId: '{}' [{}]",
                        imdb_id,
                        AppError::InternalServerError.to_string()
                    );
                    return Err(AppError::InternalServerError);
                }
            }
        } else if self.series_exists_by_imdb_id(imdb_id).await? {
            reviews_id_list = match self.series.find_one(doc! {"imdbId": imdb_id}, None).await {
                Ok(series) => series.unwrap().review_ids,
                Err(_) => {
                    error!(
                        "Error in reviews /findAllByImdbId with imdbId: '{}' [{}]",
                        imdb_id,
                        AppError::InternalServerError.to_string()
                    );
                    return Err(AppError::InternalServerError);
                }
            }
        } else {
            error!(
                "Error in reviews /findAllByImdbId with imdbId: '{}' [{}]",
                imdb_id,
                AppError::NotExists.to_string()
            );
            return Err(AppError::NotExists);
        }

        let cursor = self
            .reviews
            .find(doc! { "_id": { "$in": reviews_id_list } }, None)
            .await
            .ok()
            .expect("Error finding all reviews");

        let review_list: Vec<ReviewResponse> = cursor
            .map(|review| ReviewResponse::try_from(review.unwrap()))
            .try_collect()
            .await
            .ok()
            .expect("Error collecting reviews");

        if review_list.is_empty() {
            warn!(
                "Warn in reviews /findAllByImdbId [{}]",
                AppError::Empty.to_string()
            );
            return Err(AppError::Empty);
        }

        Ok(review_list)
    }

    async fn find_review_by_id(&self, id: &str) -> Result<ReviewResponse, AppError> {
        info!("GET reviews /findById with id: '{}' executed", id);
        let obj_id = ObjectId::from_str(id)?;
        let review: ReviewResponse = match self.reviews.find_one(doc! {"_id": obj_id}, None).await {
            Ok(Some(review)) => ReviewResponse::try_from(review).unwrap(),
            Ok(None) => {
                warn!(
                    "Warn in reviews /findById with id: '{}' [{}]",
                    id,
                    AppError::NotFound.to_string()
                );
                return Err(AppError::NotFound);
            }
            Err(_) => {
                error!(
                    "Error in reviews /findById with id: '{}' [{}]",
                    id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        Ok(review)
    }

    async fn create_review(
        &self,
        review: Review,
        imdb_id: &str,
    ) -> Result<Map<String, Value>, AppError> {
        info!("POST reviews /new executed");
        let mut map_result: Map<String, Value> = Map::new();

        if self.movie_exists_by_imdb_id(imdb_id).await? {
            let movie = match self.movies.find_one(doc! {"imdbId": imdb_id}, None).await {
                Ok(Some(movie)) => movie,
                Ok(None) => {
                    warn!(
                        "Warn finding movie in reviews /findAllByImdbId with imdbId: '{}' [{}]",
                        imdb_id,
                        AppError::NotExists.to_string()
                    );
                    return Err(AppError::NotExists);
                }
                Err(_) => {
                    error!(
                        "Error finding movie in reviews /findAllByImdbId with imdbId: '{}' [{}]",
                        imdb_id,
                        AppError::InternalServerError.to_string()
                    );
                    return Err(AppError::InternalServerError);
                }
            };
            let result = self
                .reviews
                .insert_one(review, None)
                .await
                .ok()
                .expect(format!("Error creating review with imdbId: '{}'", imdb_id).as_str());

            self.movies
                .update_one(
                    doc! { "_id": movie._id },
                    doc! { "$push": { "reviewIds": &result.inserted_id } },
                    None,
                )
                .await
                .ok()
                .expect(
                    format!(
                        "Error updating movie reviewIds field with imdbId: '{}'",
                        imdb_id
                    )
                    .as_str(),
                );

            map_result.insert(
                "message".to_string(),
                Value::String(
                    format!(
                        "Review was successfully created. (id: '{}')",
                        result.inserted_id.as_object_id().unwrap().to_string()
                    )
                    .to_string(),
                ),
            );
        } else if self.series_exists_by_imdb_id(imdb_id).await? {
            let series = match self.series.find_one(doc! {"imdbId": imdb_id}, None).await {
                Ok(Some(series)) => series,
                Ok(None) => {
                    warn!(
                        "Warn finding series in reviews /findAllByImdbId with imdbId: '{}' [{}]",
                        imdb_id,
                        AppError::NotExists.to_string()
                    );
                    return Err(AppError::NotExists);
                }
                Err(_) => {
                    error!(
                        "Error finding series in reviews /findAllByImdbId with imdbId: '{}' [{}]",
                        imdb_id,
                        AppError::InternalServerError.to_string()
                    );
                    return Err(AppError::InternalServerError);
                }
            };
            let result = self
                .reviews
                .insert_one(review, None)
                .await
                .ok()
                .expect(format!("Error creating review with imdbId: '{}'", imdb_id).as_str());

            self.series
                .update_one(
                    doc! { "_id": series._id },
                    doc! { "$push": { "reviewIds": &result.inserted_id } },
                    None,
                )
                .await
                .ok()
                .expect(
                    format!(
                        "Error updating series reviewIds field with imdbId: '{}'",
                        imdb_id
                    )
                    .as_str(),
                );

            map_result.insert(
                "message".to_string(),
                Value::String(
                    format!(
                        "Review was successfully created. (id: '{}')",
                        result.inserted_id.as_object_id().unwrap().to_string()
                    )
                    .to_string(),
                ),
            );
        } else {
            error!(
                "Error finding movie and series in reviews /findAllByImdbId with imdbId: '{}' [{}]",
                imdb_id,
                AppError::NotExists.to_string()
            );
            return Err(AppError::NotExists);
        }
        Ok(map_result)
    }

    async fn movie_exists_by_review_id(
        &self,
        review_id: ObjectId,
    ) -> Result<(bool, Option<ObjectId>), AppError> {
        let res: (bool, Option<ObjectId>) = match self
            .movies
            .find_one(doc! { "reviewIds": review_id }, None)
            .await
        {
            Ok(Some(movie)) => (true, Some(movie._id)),
            Ok(None) => (false, None),
            Err(_) => {
                error!(
                    "Error checking if movie exists with review id: '{}' [{}]",
                    review_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        Ok(res)
    }

    async fn series_exists_by_review_id(
        &self,
        review_id: ObjectId,
    ) -> Result<(bool, Option<ObjectId>), AppError> {
        let res: (bool, Option<ObjectId>) = match self
            .series
            .find_one(doc! { "reviewIds": review_id }, None)
            .await
        {
            Ok(Some(series)) => (true, Some(series._id)),
            Ok(None) => (false, None),
            Err(_) => {
                error!(
                    "Error checking if series exists with review id: '{}' [{}]",
                    review_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        Ok(res)
    }

    async fn delete_review(&self, id: &str) -> Result<Map<String, Value>, AppError> {
        info!("DELETE reviews /delete with id: '{}' executed", id);
        let obj_id = ObjectId::from_str(id)?;
        let del_result = match self.reviews.delete_one(doc! {"_id": obj_id}, None).await {
            Ok(res) => res,
            Err(_) => {
                error!(
                    "Error in reviews /delete with id: '{}' [{}]",
                    obj_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };

        let exists_movie_tup = self.movie_exists_by_review_id(obj_id).await?;
        let exists_series_tup = self.series_exists_by_review_id(obj_id).await?;
        if exists_movie_tup.0 {
            self.movies
                .update_one(
                    doc! { "_id": exists_movie_tup.1.unwrap() },
                    doc! { "$pull": { "reviewIds": obj_id } },
                    None,
                )
                .await
                .ok()
                .expect(
                    format!(
                        "Error removing review id from movie reviewsIds field with id: '{}'",
                        id
                    )
                    .as_str(),
                );
        } else if exists_series_tup.0 {
            self.series
                .update_one(
                    doc! { "_id": exists_series_tup.1.unwrap() },
                    doc! { "$pull": { "reviewIds": obj_id } },
                    None,
                )
                .await
                .ok()
                .expect(
                    format!(
                        "Error removing review id from series reviewsIds field with id: '{}'",
                        id
                    )
                    .as_str(),
                );
        } else {
            error!(
                "Error finding movie and series in reviews /delete with id: '{}' [{}]",
                id,
                AppError::NotExists.to_string()
            );
            return Err(AppError::NotExists);
        }

        let mut map_result: Map<String, Value> = Map::new();
        if del_result.deleted_count > 0 {
            map_result.insert(
                "message".to_string(),
                Value::String(
                    format!("Review with id: '{}' was successfully deleted", id).to_string(),
                ),
            );
        } else {
            warn!(
                "Warn in review /delete with id: '{}' [{}]",
                obj_id,
                AppError::NotExists.to_string()
            );
            return Err(AppError::NotExists);
        }
        Ok(map_result)
    }

    async fn update_review(
        &self,
        id: &str,
        review: ReviewUpdate,
    ) -> Result<Map<String, Value>, AppError> {
        info!("PUT reviews /update with id: '{}' executed", id);
        let obj_id = ObjectId::from_str(id)?;
        match self.reviews.find_one(doc! { "_id": obj_id }, None).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                warn!(
                    "Warn in reviews /update with id: '{}' [{}]",
                    obj_id,
                    AppError::NotExists.to_string()
                );
                return Err(AppError::NotExists);
            }
            Err(_) => {
                error!(
                    "Error in reviews /update with id: '{}' [{}]",
                    obj_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        let result = self
            .reviews
            .update_one(
                doc! { "_id": obj_id },
                doc! {
                "$set": doc! {
                    "title": review.title,
                    "rating": review.rating,
                    "body": review.body,
                    "updatedAt": DateTime::now(),
                }},
                None,
            )
            .await
            .ok()
            .expect(format!("Error updating review with id: '{}'", id).as_str());
        let mut map_result: Map<String, Value> = Map::new();
        map_result.insert(
            "message".to_string(),
            Value::String(if result.modified_count != 0 {
                format!("Review with id: '{}' was successfully updated", id)
            } else {
                "Fields have the same value, no update was performed".to_string()
            }),
        );
        Ok(map_result)
    }

    async fn patch_review(
        &self,
        id: &str,
        field: &str,
        val: &str,
    ) -> Result<Map<String, Value>, AppError> {
        info!("PATCH reviews /patch with id: '{}' executed", id);
        let fields_vec: Vec<&str> = vec!["title", "rating", "body"];
        let obj_id = ObjectId::from_str(id)?;
        if !fields_vec.contains(&field) {
            warn!(
                "Warn in reviews /patch with id: '{}' [{}]",
                obj_id,
                AppError::FieldNotAllowed.to_string()
            );
            return Err(AppError::FieldNotAllowed);
        }
        match self.reviews.find_one(doc! { "_id": obj_id }, None).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                warn!(
                    "Warn in reviews /patch with id: '{}' [{}]",
                    obj_id,
                    AppError::NotExists.to_string()
                );
                return Err(AppError::NotExists);
            }
            Err(_) => {
                error!(
                    "Error in reviews /patch with id: '{}' [{}]",
                    obj_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        let result = self
            .reviews
            .update_one(
                doc! { "_id": obj_id },
                doc! {
                "$set": doc! {
                    field: to_bson(val).unwrap(),
                    "updatedAt": DateTime::now(),
                }},
                None,
            )
            .await
            .ok()
            .expect(format!("Error patching reviews with id: '{}'", id).as_str());
        let mut map_result: Map<String, Value> = Map::new();
        map_result.insert(
            "message".to_string(),
            Value::String(if result.modified_count != 0 {
                format!(
                    "Review {} with id: '{}' was successfully patched",
                    field, id
                )
            } else {
                "Field has the same value, no patch was performed".to_string()
            }),
        );
        Ok(map_result)
    }
}
