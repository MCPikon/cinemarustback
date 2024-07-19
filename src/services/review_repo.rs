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

#[cfg_attr(test, mockall::automock)]
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

#[cfg(test)]
mod tests {

    use super::*;

    // Auxiliary Functions

    fn build_review_update_mock() -> ReviewUpdate {
        ReviewUpdate {
            title: "El padrino es una obra de arte.".to_string(),
            rating: 4,
            body: "En esta nueva entrega del padrino vemos a un Michael Corleone mucho más maduro."
                .to_string(),
        }
    }

    // Unit Tests

    #[actix_web::test]
    async fn test_find_all_reviews_ok() {
        let mut mock = MockReviewRepository::new();

        mock.expect_find_all_reviews().returning(|_, _| {
            let mut result_map = serde_json::Map::new();
            let review = ReviewResponse {
                _id: ObjectId::new(),
                title: "La mejor película de la historia".to_string(),
                rating: 5,
                body: "Esta película es una obra de arte, es perfecta".to_string(),
                created_at: DateTime::now(),
                updated_at: DateTime::now(),
            };
            result_map.insert(
                "reviews".to_string(),
                serde_json::to_value(vec![review]).unwrap(),
            );
            result_map.insert("currentPage".to_string(), serde_json::to_value(1).unwrap());
            result_map.insert("totalItems".to_string(), serde_json::to_value(1).unwrap());
            result_map.insert("totalPages".to_string(), serde_json::to_value(1).unwrap());
            Ok(result_map)
        });

        let result = mock.find_all_reviews(Some(1), Some(10)).await;
        assert!(result.is_ok());

        let map = result.unwrap();
        assert_eq!(map.get("currentPage").unwrap(), 1);
        assert_eq!(map.get("totalItems").unwrap(), 1);
        assert_eq!(map.get("totalPages").unwrap(), 1);

        let review_list = map.get("reviews").unwrap().as_array().unwrap();
        assert_eq!(review_list.len(), 1);
        assert_eq!(
            review_list[0].get("title").unwrap(),
            "La mejor película de la historia"
        );
    }

    #[actix_web::test]
    async fn test_find_all_reviews_empty_list() {
        let mut mock = MockReviewRepository::new();

        mock.expect_find_all_reviews()
            .returning(|_, _| Err(AppError::Empty));

        let result = mock.find_all_reviews(Some(1), Some(10)).await;
        assert!(result.is_err_and(|err| err == AppError::Empty));
    }

    #[actix_web::test]
    async fn test_find_all_reviews_internal_server_error() {
        let mut mock = MockReviewRepository::new();

        mock.expect_find_all_reviews()
            .returning(|_, _| Err(AppError::InternalServerError));

        let result = mock.find_all_reviews(Some(1), Some(10)).await;
        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_find_all_reviews_by_imdb_id_ok() {
        let mut mock = MockReviewRepository::new();

        mock.expect_find_all_reviews_by_imdb_id().returning(|_| {
            let review = ReviewResponse {
                _id: ObjectId::new(),
                title: "La mejor película de la historia".to_string(),
                rating: 5,
                body: "Esta película es una obra de arte, es perfecta".to_string(),
                created_at: DateTime::now(),
                updated_at: DateTime::now(),
            };
            Ok(vec![review])
        });

        let result = mock.find_all_reviews_by_imdb_id("tt1234").await;
        assert!(result.is_ok());

        let review_list = result.unwrap();
        assert_eq!(review_list.len(), 1);
        assert_eq!(review_list[0].title, "La mejor película de la historia");
    }

    #[actix_web::test]
    async fn test_find_all_reviews_by_imdb_id_wrong_imdb_id() {
        let mut mock = MockReviewRepository::new();

        mock.expect_find_all_reviews_by_imdb_id()
            .returning(|_| Err(AppError::WrongImdbId));

        let result = mock.find_all_reviews_by_imdb_id("tt1234").await;

        assert!(result.is_err_and(|err| err == AppError::WrongImdbId));
    }

    #[actix_web::test]
    async fn test_find_all_reviews_by_imdb_id_internal_server_error() {
        let mut mock = MockReviewRepository::new();

        mock.expect_find_all_reviews_by_imdb_id()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.find_all_reviews_by_imdb_id("tt1234").await;

        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_find_all_reviews_by_imdb_id_not_exists() {
        let mut mock = MockReviewRepository::new();

        mock.expect_find_all_reviews_by_imdb_id()
            .returning(|_| Err(AppError::NotExists));

        let result = mock.find_all_reviews_by_imdb_id("tt1234").await;

        assert!(result.is_err_and(|err| err == AppError::NotExists));
    }

    #[actix_web::test]
    async fn test_find_all_reviews_by_imdb_id_empty() {
        let mut mock = MockReviewRepository::new();

        mock.expect_find_all_reviews_by_imdb_id()
            .returning(|_| Err(AppError::Empty));

        let result = mock.find_all_reviews_by_imdb_id("tt1234").await;

        assert!(result.is_err_and(|err| err == AppError::Empty));
    }

    #[actix_web::test]
    async fn test_find_review_by_id_ok() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_review_by_id().returning(move |_| {
            let review = ReviewResponse {
                _id: oid,
                title: "La mejor película de la historia".to_string(),
                rating: 5,
                body: "Esta película es una obra de arte, es perfecta".to_string(),
                created_at: DateTime::now(),
                updated_at: DateTime::now(),
            };
            Ok(review)
        });

        let result = mock.find_review_by_id(oid.to_string().as_str()).await;
        assert!(result.is_ok_and(|res| {
            res.title == "La mejor película de la historia" && res.rating == 5
        }));
    }

    #[actix_web::test]
    async fn test_find_review_by_id_cannot_parse_object_id() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_review_by_id()
            .returning(|_| Err(AppError::CannotParseObjId));

        let result = mock.find_review_by_id(oid.to_string().as_str()).await;

        assert!(result.is_err_and(|err| err == AppError::CannotParseObjId));
    }

    #[actix_web::test]
    async fn test_find_review_by_id_cannot_not_found() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_review_by_id()
            .returning(|_| Err(AppError::NotFound));

        let result = mock.find_review_by_id(oid.to_string().as_str()).await;

        assert!(result.is_err_and(|err| err == AppError::NotFound));
    }

    #[actix_web::test]
    async fn test_find_review_by_id_cannot_internal_server_error() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_review_by_id()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.find_review_by_id(oid.to_string().as_str()).await;

        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_create_review_ok() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let review = Review {
            _id: oid,
            title: "La mejor película de la historia".to_string(),
            rating: 5,
            body: "Esta película es una obra de arte, es perfecta".to_string(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        };

        mock.expect_create_review().returning(|review, _| {
            let mut map_result: Map<String, Value> = Map::new();
            map_result.insert(
                "message".to_string(),
                Value::String(format!(
                    "Review was successfully created. (id: '{}')",
                    review._id
                )),
            );
            Ok(map_result)
        });

        let result = mock.create_review(review, "tt12345").await;

        assert!(result.is_ok_and(
            |map| map["message"] == format!("Review was successfully created. (id: '{}')", oid)
        ));
    }

    #[actix_web::test]
    async fn test_create_review_not_exists() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let review = Review {
            _id: oid,
            title: "La mejor película de la historia".to_string(),
            rating: 5,
            body: "Esta película es una obra de arte, es perfecta".to_string(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        };

        mock.expect_create_review()
            .returning(|_, _| Err(AppError::NotExists));

        let result = mock.create_review(review, "tt12345").await;

        assert!(result.is_err_and(|err| err == AppError::NotExists));
    }

    #[actix_web::test]
    async fn test_create_review_internal_server_error() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let review = Review {
            _id: oid,
            title: "La mejor película de la historia".to_string(),
            rating: 5,
            body: "Esta película es una obra de arte, es perfecta".to_string(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        };

        mock.expect_create_review()
            .returning(|_, _| Err(AppError::InternalServerError));

        let result = mock.create_review(review, "tt12345").await;

        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_movie_exists_by_imdb_id_true() {
        let mut mock = MockReviewRepository::new();
        let movie_oid = ObjectId::new();

        mock.expect_movie_exists_by_review_id()
            .returning(move |_| Ok((true, Some(movie_oid))));

        let result = mock.movie_exists_by_review_id(ObjectId::new()).await;
        assert!(result.is_ok_and(|tup| { tup.0 && tup.1.unwrap() == movie_oid }));
    }

    #[actix_web::test]
    async fn test_movie_exists_by_imdb_id_false() {
        let mut mock = MockReviewRepository::new();

        mock.expect_movie_exists_by_review_id()
            .returning(|_| Ok((false, None)));

        let result = mock.movie_exists_by_review_id(ObjectId::new()).await;
        assert!(result.is_ok_and(|tup| { !tup.0 && tup.1 == None }));
    }

    #[actix_web::test]
    async fn test_movie_exists_by_imdb_id_internal_server_error() {
        let mut mock = MockReviewRepository::new();

        mock.expect_movie_exists_by_review_id()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.movie_exists_by_review_id(ObjectId::new()).await;

        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_series_exists_by_imdb_id_true() {
        let mut mock = MockReviewRepository::new();
        let series_oid = ObjectId::new();

        mock.expect_series_exists_by_review_id()
            .returning(move |_| Ok((true, Some(series_oid))));

        let result = mock.series_exists_by_review_id(ObjectId::new()).await;
        assert!(result.is_ok_and(|tup| { tup.0 && tup.1.unwrap() == series_oid }));
    }

    #[actix_web::test]
    async fn test_series_exists_by_imdb_id_false() {
        let mut mock = MockReviewRepository::new();

        mock.expect_series_exists_by_review_id()
            .returning(|_| Ok((false, None)));

        let result = mock.series_exists_by_review_id(ObjectId::new()).await;
        assert!(result.is_ok_and(|tup| { !tup.0 && tup.1 == None }));
    }

    #[actix_web::test]
    async fn test_series_exists_by_imdb_id_internal_server_error() {
        let mut mock = MockReviewRepository::new();

        mock.expect_series_exists_by_review_id()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.series_exists_by_review_id(ObjectId::new()).await;

        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_delete_review_ok() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let del_msg = format!("Review with id: '{}' was successfully deleted", oid);

        mock.expect_delete_review().returning({
            let msg = del_msg.clone();
            move |_| {
                let mut map_result: Map<String, Value> = Map::new();
                map_result.insert("message".to_string(), Value::String(msg.clone()));
                Ok(map_result)
            }
        });

        let result = mock.delete_review(oid.to_string().as_str()).await;

        assert!(result.is_ok_and(|map| map["message"] == del_msg));
    }

    #[actix_web::test]
    async fn test_delete_review_cannot_parse_object_id() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();

        mock.expect_delete_review()
            .returning(|_| Err(AppError::CannotParseObjId));

        let result = mock.delete_review(oid.to_string().as_str()).await;

        assert!(result.is_err_and(|err| err == AppError::CannotParseObjId));
    }

    #[actix_web::test]
    async fn test_delete_review_not_exists() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();

        mock.expect_delete_review()
            .returning(|_| Err(AppError::NotExists));

        let result = mock.delete_review(oid.to_string().as_str()).await;

        assert!(result.is_err_and(|err| err == AppError::NotExists));
    }

    #[actix_web::test]
    async fn test_delete_review_internal_server_error() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();

        mock.expect_delete_review()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.delete_review(oid.to_string().as_str()).await;

        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_update_review_ok() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let review_up = build_review_update_mock();
        let upd_msg = format!("Review with id: '{}' was successfully updated", oid);

        mock.expect_update_review().returning({
            let msg = upd_msg.clone();
            move |_, _| {
                let mut map_result: Map<String, Value> = Map::new();
                map_result.insert("message".to_string(), Value::String(msg.clone()));
                Ok(map_result)
            }
        });

        let result = mock
            .update_review(oid.to_string().as_str(), review_up)
            .await;

        assert!(result.is_ok_and(|map| map["message"] == upd_msg));
    }

    #[actix_web::test]
    async fn test_update_review_cannot_parse_object_id() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let review_up = build_review_update_mock();

        mock.expect_update_review()
            .returning(|_, _| Err(AppError::CannotParseObjId));

        let result = mock
            .update_review(oid.to_string().as_str(), review_up)
            .await;

        assert!(result.is_err_and(|err| err == AppError::CannotParseObjId));
    }

    #[actix_web::test]
    async fn test_update_review_not_exists() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let review_up = build_review_update_mock();

        mock.expect_update_review()
            .returning(|_, _| Err(AppError::NotExists));

        let result = mock
            .update_review(oid.to_string().as_str(), review_up)
            .await;

        assert!(result.is_err_and(|err| err == AppError::NotExists));
    }

    #[actix_web::test]
    async fn test_update_review_internal_server_error() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let review_up = build_review_update_mock();

        mock.expect_update_review()
            .returning(|_, _| Err(AppError::InternalServerError));

        let result = mock
            .update_review(oid.to_string().as_str(), review_up)
            .await;

        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_patch_review_ok() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let field = "title";
        let val = "Jedis vs Sith, la nueva serie de Star Wars lo peta";
        let pt_msg = format!(
            "Review {} with id: '{}' was successfully patched",
            field, oid
        );

        mock.expect_patch_review().returning({
            let msg = pt_msg.clone();
            move |_, _, _| {
                let mut map_result: Map<String, Value> = Map::new();
                map_result.insert("message".to_string(), Value::String(msg.clone()));
                Ok(map_result)
            }
        });

        let result = mock
            .patch_review(oid.to_string().as_str(), field, val)
            .await;

        assert!(result.is_ok_and(|map| map["message"] == pt_msg));
    }

    #[actix_web::test]
    async fn test_patch_review_cannot_parse_object_id() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let field = "title";
        let val = "Jedis vs Sith, la nueva serie de Star Wars lo peta";

        mock.expect_patch_review()
            .returning(|_, _, _| Err(AppError::CannotParseObjId));

        let result = mock
            .patch_review(oid.to_string().as_str(), field, val)
            .await;

        assert!(result.is_err_and(|err| err == AppError::CannotParseObjId));
    }

    #[actix_web::test]
    async fn test_patch_review_field_not_allowed() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let field = "title";
        let val = "Jedis vs Sith, la nueva serie de Star Wars lo peta";

        mock.expect_patch_review()
            .returning(|_, _, _| Err(AppError::FieldNotAllowed));

        let result = mock
            .patch_review(oid.to_string().as_str(), field, val)
            .await;

        assert!(result.is_err_and(|err| err == AppError::FieldNotAllowed));
    }

    #[actix_web::test]
    async fn test_patch_review_not_exists() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let field = "title";
        let val = "Jedis vs Sith, la nueva serie de Star Wars lo peta";

        mock.expect_patch_review()
            .returning(|_, _, _| Err(AppError::NotExists));

        let result = mock
            .patch_review(oid.to_string().as_str(), field, val)
            .await;

        assert!(result.is_err_and(|err| err == AppError::NotExists));
    }

    #[actix_web::test]
    async fn test_patch_review_internal_server_error() {
        let mut mock = MockReviewRepository::new();
        let oid = ObjectId::new();
        let field = "title";
        let val = "Jedis vs Sith, la nueva serie de Star Wars lo peta";

        mock.expect_patch_review()
            .returning(|_, _, _| Err(AppError::InternalServerError));

        let result = mock
            .patch_review(oid.to_string().as_str(), field, val)
            .await;

        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }
}
