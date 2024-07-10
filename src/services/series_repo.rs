use std::str::FromStr;

use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use log::{error, info, warn};
use mongodb::{
    bson::{doc, oid::ObjectId, to_bson, Regex},
    options::{CountOptions, FindOptions},
};
use serde_json::{Map, Value};

use super::{db::Database, movie_repo::MovieRepository};

use crate::{
    error::AppError,
    models::series::{Series, SeriesRequest, SeriesResponse},
};

lazy_static! {
    static ref RE_IMDB_ID: regex::Regex = regex::Regex::new(r"^tt\d+$").unwrap();
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SeriesRepository {
    async fn find_all_series(
        &self,
        title: Option<String>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<Map<String, Value>, AppError>;
    async fn find_series_by_id(&self, id: &str) -> Result<Series, AppError>;
    async fn find_series_by_imdb_id(&self, imdb_id: &str) -> Result<Series, AppError>;
    async fn create_series(&self, series: Series) -> Result<Map<String, Value>, AppError>;
    async fn delete_series(&self, id: &str) -> Result<Map<String, Value>, AppError>;
    async fn series_exists_by_imdb_id(&self, imdb_id: &str) -> Result<bool, AppError>;
    async fn update_series(
        &self,
        id: &str,
        series: SeriesRequest,
    ) -> Result<Map<String, Value>, AppError>;
    async fn patch_series(
        &self,
        id: &str,
        field: &str,
        val: &str,
    ) -> Result<Map<String, Value>, AppError>;
}

#[async_trait]
impl SeriesRepository for Database {
    async fn find_all_series(
        &self,
        title: Option<String>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<Map<String, Value>, AppError> {
        info!("GET series /findAll executed");
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
        let filter = match title {
            None => None,
            Some(title) => {
                let regex = Regex {
                    pattern: format!("{}", title),
                    options: String::new(),
                };
                doc! {"title": { "$regex": regex, "$options": "i" }}.into()
            }
        };

        let total_items = self
            .series
            .count_documents(filter.clone(), CountOptions::default())
            .await
            .ok()
            .expect("Error counting total of series");
        let total_pages = (total_items as f64 / page_size as f64).ceil() as u64;

        let options = FindOptions::builder()
            .skip((page_num * page_size) as u64)
            .limit(page_size as i64)
            .build();

        let cursor = self
            .series
            .find(filter, options)
            .await
            .ok()
            .expect("Error finding all series");

        let series_list: Vec<SeriesResponse> = cursor
            .map(|series| SeriesResponse::try_from(series.unwrap()))
            .try_collect()
            .await
            .ok()
            .expect("Error collecting series");

        if series_list.is_empty() {
            warn!("Warn in series /findAll [{}]", AppError::Empty.to_string());
            return Err(AppError::Empty);
        }

        result_map.insert(
            "series".to_string(),
            serde_json::to_value(series_list).unwrap(),
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

    async fn find_series_by_id(&self, id: &str) -> Result<Series, AppError> {
        info!("GET series /findById with id: '{}' executed", id);
        let obj_id = ObjectId::from_str(id)?;
        let series: Series = match self.series.find_one(doc! {"_id": obj_id}, None).await {
            Ok(Some(series)) => series,
            Ok(None) => {
                warn!(
                    "Warn in series /findById with id: '{}' [{}]",
                    id,
                    AppError::NotFound.to_string()
                );
                return Err(AppError::NotFound);
            }
            Err(_) => {
                error!(
                    "Error in series /findById with id: '{}' [{}]",
                    id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        Ok(series)
    }

    async fn find_series_by_imdb_id(&self, imdb_id: &str) -> Result<Series, AppError> {
        info!("GET series /findByImdbId with id: '{}' executed", imdb_id);
        if !RE_IMDB_ID.is_match(imdb_id) {
            error!(
                "Error in series /findByImdbId with imdbId: '{}' [{}]",
                imdb_id,
                AppError::WrongImdbId.to_string()
            );
            return Err(AppError::WrongImdbId);
        }

        let series: Series = match self.series.find_one(doc! {"imdbId": imdb_id}, None).await {
            Ok(Some(series)) => series,
            Ok(None) => {
                warn!(
                    "Warn in series /findByImdbId with imdbId: '{}' [{}]",
                    imdb_id,
                    AppError::NotFound.to_string()
                );
                return Err(AppError::NotFound);
            }
            Err(_) => {
                error!(
                    "Error in series /findByImdbId with imdbId: '{}' [{}]",
                    imdb_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        Ok(series)
    }

    async fn create_series(&self, series: Series) -> Result<Map<String, Value>, AppError> {
        info!("POST series /new executed");
        if self
            .series_exists_by_imdb_id(series.imdb_id.clone().as_str())
            .await?
        {
            warn!(
                "Warn in series /new [{}]",
                AppError::AlreadyExists.to_string()
            );
            return Err(AppError::AlreadyExists);
        }
        if self
            .movie_exists_by_imdb_id(series.imdb_id.clone().as_str())
            .await?
        {
            warn!(
                "Warn in series /new [{}]",
                AppError::AlreadyExists.to_string()
            );
            return Err(AppError::AlreadyExists);
        }
        let result =
            self.series.insert_one(&series, None).await.ok().expect(
                format!("Error creating series with imdbId: '{}'", series.imdb_id).as_str(),
            );

        let mut map_result: Map<String, Value> = Map::new();
        map_result.insert(
            "message".to_string(),
            Value::String(
                format!(
                    "Series was successfully created. (id: '{}')",
                    result.inserted_id.as_object_id().unwrap().to_string()
                )
                .to_string(),
            ),
        );
        Ok(map_result)
    }

    async fn delete_series(&self, id: &str) -> Result<Map<String, Value>, AppError> {
        info!("DELETE series /delete with id: '{}' executed", id);
        let obj_id = ObjectId::from_str(id)?;
        let del_result = match self.series.delete_one(doc! {"_id": obj_id}, None).await {
            Ok(res) => res,
            Err(_) => {
                error!(
                    "Error in series /delete with id: '{}' [{}]",
                    obj_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        let mut map_result: Map<String, Value> = Map::new();
        if del_result.deleted_count > 0 {
            map_result.insert(
                "message".to_string(),
                Value::String(
                    format!("Series with id: '{}' was successfully deleted", id).to_string(),
                ),
            );
        } else {
            warn!(
                "Warn in series /delete with id: '{}' [{}]",
                obj_id,
                AppError::NotExists.to_string()
            );
            return Err(AppError::NotExists);
        }
        Ok(map_result)
    }

    async fn series_exists_by_imdb_id(&self, imdb_id: &str) -> Result<bool, AppError> {
        let exists: bool = match self.series.find_one(doc! { "imdbId": imdb_id }, None).await {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => {
                error!(
                    "Error checking if series exists with imdbId: '{}' [{}]",
                    imdb_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        Ok(exists)
    }

    async fn update_series(
        &self,
        id: &str,
        series: SeriesRequest,
    ) -> Result<Map<String, Value>, AppError> {
        info!("PUT series /update with id: '{}' executed", id);
        let obj_id = ObjectId::from_str(id)?;
        let series_founded: Series = match self.series.find_one(doc! { "_id": obj_id }, None).await
        {
            Ok(Some(series)) => series,
            Ok(None) => {
                warn!(
                    "Warn in series /update with id: '{}' [{}]",
                    obj_id,
                    AppError::NotExists.to_string()
                );
                return Err(AppError::NotExists);
            }
            Err(_) => {
                error!(
                    "Error in series /update with id: '{}' [{}]",
                    obj_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        let exists_imdb_id_movie: bool = self.movie_exists_by_imdb_id(&series.imdb_id).await?;
        let exists_imdb_id_series: bool = self.series_exists_by_imdb_id(&series.imdb_id).await?;
        if (exists_imdb_id_movie || exists_imdb_id_series)
            && series_founded.imdb_id != series.imdb_id
        {
            error!(
                "Error in series /update with id: '{}' [{}]",
                obj_id,
                AppError::ImdbIdInUse.to_string()
            );
            return Err(AppError::ImdbIdInUse);
        }
        let result = self
            .series
            .update_one(
                doc! { "_id": obj_id },
                doc! {
                "$set": doc! {
                    "imdbId": series.imdb_id,
                    "title": series.title,
                    "overview": series.overview,
                    "numberOfSeasons": series.number_of_seasons,
                    "creator": series.creator,
                    "releaseDate": series.release_date,
                    "trailerLink": series.trailer_link,
                    "genres": series.genres,
                    "seasonList": to_bson(&series.season_list).unwrap(),
                    "poster": series.poster,
                    "backdrop": series.backdrop
                }},
                None,
            )
            .await
            .ok()
            .expect(format!("Error updating series with id: '{}'", id).as_str());
        let mut map_result: Map<String, Value> = Map::new();
        map_result.insert(
            "message".to_string(),
            Value::String(if result.modified_count != 0 {
                format!("Series with id: '{}' was successfully updated", id)
            } else {
                "Fields have the same value, no update was performed".to_string()
            }),
        );
        Ok(map_result)
    }

    async fn patch_series(
        &self,
        id: &str,
        field: &str,
        val: &str,
    ) -> Result<Map<String, Value>, AppError> {
        info!("PATCH series /patch with id: '{}' executed", id);
        let fields_vec: Vec<&str> = vec![
            "imdbId",
            "title",
            "overview",
            "numberOfSeasons",
            "creator",
            "releaseDate",
            "trailerLink",
            "genres",
            "seasonList",
            "poster",
            "backdrop",
        ];
        let obj_id = ObjectId::from_str(id)?;
        if !fields_vec.contains(&field) {
            warn!(
                "Warn in series /patch with id: '{}' [{}]",
                obj_id,
                AppError::FieldNotAllowed.to_string()
            );
            return Err(AppError::FieldNotAllowed);
        }
        let series_founded: Series = match self.series.find_one(doc! { "_id": obj_id }, None).await
        {
            Ok(Some(series)) => series,
            Ok(None) => {
                warn!(
                    "Warn in series /patch with id: '{}' [{}]",
                    obj_id,
                    AppError::NotExists.to_string()
                );
                return Err(AppError::NotExists);
            }
            Err(_) => {
                error!(
                    "Error in series /patch with id: '{}' [{}]",
                    obj_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        if field == "imdbId" {
            if !RE_IMDB_ID.is_match(val) {
                error!(
                    "Error in series /patch with id: '{}' [{}]",
                    id,
                    AppError::WrongImdbId.to_string()
                );
                return Err(AppError::WrongImdbId);
            }
            let exists_imdb_id_movie: bool = self.movie_exists_by_imdb_id(val).await?;
            let exists_imdb_id_series: bool = self.series_exists_by_imdb_id(val).await?;
            if (exists_imdb_id_movie || exists_imdb_id_series) && series_founded.imdb_id != val {
                error!(
                    "Error in series /patch with id: '{}' [{}]",
                    obj_id,
                    AppError::ImdbIdInUse.to_string()
                );
                return Err(AppError::ImdbIdInUse);
            }
        }
        let result = self
            .series
            .update_one(
                doc! { "_id": obj_id },
                doc! {
                "$set": doc! {
                    field: to_bson(val).unwrap()
                }},
                None,
            )
            .await
            .ok()
            .expect(format!("Error patching series with id: '{}'", id).as_str());
        let mut map_result: Map<String, Value> = Map::new();
        map_result.insert(
            "message".to_string(),
            Value::String(if result.modified_count != 0 {
                format!(
                    "Series {} with id: '{}' was successfully patched",
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
    use SeriesResponse;

    use {MockSeriesRepository, SeriesRepository};

    use super::*;

    // Auxiliary Functions
    fn build_series_mock(oid: ObjectId) -> Series {
        Series {
            _id: oid,
            imdb_id: "tt12345".to_string(),
            title: "Breaking Bad".to_string(),
            overview: "Serie sobre Walter White y su imperio de la meta.".to_string(),
            number_of_seasons: 5,
            creator: "Vince Gilligan".to_string(),
            release_date: "2006-04-02".to_string(),
            trailer_link: "https://youtube.com/video/dsan21jhk1j".to_string(),
            genres: vec!["Crimen".to_string(), "Drama".to_string()],
            season_list: vec![],
            poster: "https://moviedb.com/breaking_bad/poster.jpg".to_string(),
            backdrop: "https://moviedb.com/breaking_bad/backdrop.jpg".to_string(),
            review_ids: vec![ObjectId::new()],
        }
    }

    // Unit Tests

    #[actix_web::test]
    async fn test_find_all_series_ok() {
        let mut mock = MockSeriesRepository::new();

        mock.expect_find_all_series().returning(|_, _, _| {
            let mut result_map = serde_json::Map::new();
            let series = SeriesResponse {
                imdb_id: "tt12345".to_string(),
                title: "Breaking Bad".to_string(),
                number_of_seasons: 5,
                release_date: "1990-3-4".to_string(),
                poster: "https://moviedb.com/breaking_bad/poster.jpg".to_string(),
            };
            result_map.insert(
                "series".to_string(),
                serde_json::to_value(vec![series]).unwrap(),
            );
            result_map.insert("currentPage".to_string(), serde_json::to_value(1).unwrap());
            result_map.insert("totalItems".to_string(), serde_json::to_value(1).unwrap());
            result_map.insert("totalPages".to_string(), serde_json::to_value(1).unwrap());
            Ok(result_map)
        });

        let result = mock
            .find_all_series(Some("Breaking Bad".to_string()), Some(1), Some(10))
            .await;

        let map = result.unwrap();
        assert_eq!(map.get("currentPage").unwrap(), 1);
        assert_eq!(map.get("totalItems").unwrap(), 1);
        assert_eq!(map.get("totalPages").unwrap(), 1);

        let movie_list = map.get("series").unwrap().as_array().unwrap();
        assert_eq!(movie_list.len(), 1);
        assert_eq!(movie_list[0].get("title").unwrap(), "Breaking Bad");
    }

    #[actix_web::test]
    async fn test_find_all_series_empty_list() {
        let mut mock = MockSeriesRepository::new();

        mock.expect_find_all_series()
            .returning(|_, _, _| Err(AppError::Empty));

        let result = mock.find_all_series(None, Some(1), Some(10)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::Empty);
    }

    #[actix_web::test]
    async fn test_find_all_series_internal_server_error() {
        let mut mock = MockSeriesRepository::new();

        mock.expect_find_all_series()
            .returning(|_, _, _| Err(AppError::InternalServerError));

        let result = mock.find_all_series(None, Some(1), Some(10)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::InternalServerError);
    }

    #[actix_web::test]
    async fn test_find_series_by_id_ok() {
        let mut mock = MockSeriesRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_series_by_id()
            .returning(move |_| Ok(build_series_mock(oid)));

        let result = mock.find_series_by_id(oid.to_string().as_str()).await;
        assert!(result.is_ok());

        let series = result.unwrap();
        assert_eq!(series.imdb_id, "tt12345".to_string());
        assert_eq!(series.title, "Breaking Bad".to_string());
    }

    #[actix_web::test]
    async fn test_find_series_by_id_cannot_parse_obj_id() {
        let mut mock = MockSeriesRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_series_by_id()
            .returning(|_| Err(AppError::CannotParseObjId));

        let result = mock.find_series_by_id(oid.to_string().as_str()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::CannotParseObjId)
    }

    #[actix_web::test]
    async fn test_find_series_by_id_not_found() {
        let mut mock = MockSeriesRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_series_by_id()
            .returning(|_| Err(AppError::NotFound));

        let result = mock.find_series_by_id(oid.to_string().as_str()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::NotFound)
    }

    #[actix_web::test]
    async fn test_find_series_by_id_internal_server_error() {
        let mut mock = MockSeriesRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_series_by_id()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.find_series_by_id(oid.to_string().as_str()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::InternalServerError)
    }

    #[actix_web::test]
    async fn test_find_series_by_imdb_id_ok() {
        let mut mock = MockSeriesRepository::new();
        let imdb_id = "tt12345";

        mock.expect_find_series_by_imdb_id()
            .returning(|_| Ok(build_series_mock(ObjectId::new())));

        let result = mock.find_series_by_imdb_id(&imdb_id).await;
        assert!(result.is_ok());

        let series = result.unwrap();
        assert_eq!(series.imdb_id, "tt12345".to_string());
        assert_eq!(series.title, "Breaking Bad".to_string());
    }

    #[actix_web::test]
    async fn test_find_series_by_imdb_id_wrong_imdb_id() {
        let mut mock = MockSeriesRepository::new();
        let imdb_id = "tf12asD5";

        mock.expect_find_series_by_imdb_id()
            .returning(|_| Err(AppError::WrongImdbId));

        let result = mock.find_series_by_imdb_id(&imdb_id).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::WrongImdbId);
    }

    #[actix_web::test]
    async fn test_find_series_by_imdb_id_not_found() {
        let mut mock = MockSeriesRepository::new();
        let imdb_id = "tt54321";

        mock.expect_find_series_by_imdb_id()
            .returning(|_| Err(AppError::NotFound));

        let result = mock.find_series_by_imdb_id(&imdb_id).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::NotFound);
    }

    #[actix_web::test]
    async fn test_find_series_by_imdb_internal_server_error() {
        let mut mock = MockSeriesRepository::new();
        let imdb_id = "tt54321";

        mock.expect_find_series_by_imdb_id()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.find_series_by_imdb_id(&imdb_id).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::InternalServerError);
    }

    #[actix_web::test]
    async fn test_create_series_ok() {
        let mut mock = MockSeriesRepository::new();
        let oid = ObjectId::new();
        let series = build_series_mock(oid);

        mock.expect_create_series().returning(|series| {
            let mut map_result: Map<String, Value> = Map::new();
            map_result.insert(
                "message".to_string(),
                Value::String(
                    format!(
                        "Series was successfully created. (id: '{}')",
                        series._id.to_string()
                    )
                    .to_string(),
                ),
            );
            Ok(map_result)
        });

        let result = mock.create_series(series).await;
        assert!(result.is_ok());

        let map = result.unwrap();
        assert_eq!(
            map["message"],
            format!(
                "Series was successfully created. (id: '{}')",
                oid.to_string()
            )
            .to_string()
        );
    }

    #[actix_web::test]
    async fn test_create_series_already_exists() {
        let mut mock = MockSeriesRepository::new();
        let oid = ObjectId::new();
        let series = build_series_mock(oid);

        mock.expect_create_series()
            .returning(|_| Err(AppError::AlreadyExists));

        let result = mock.create_series(series).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::AlreadyExists);
    }

    #[actix_web::test]
    async fn test_create_series_internal_server_error() {
        let mut mock = MockSeriesRepository::new();
        let oid = ObjectId::new();
        let series = build_series_mock(oid);

        mock.expect_create_series()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.create_series(series).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::InternalServerError);
    }

    #[actix_web::test]
    async fn test_delete_series_ok() {
        let mut mock = MockSeriesRepository::new();
        let oid = ObjectId::new();
        let parsed_oid = oid.to_string();
        let str_oid = parsed_oid.as_str();

        mock.expect_delete_series().returning(move |str_oid| {
            let mut map_result: Map<String, Value> = Map::new();
            map_result.insert(
                "message".to_string(),
                Value::String(
                    format!("Series with id: '{}' was successfully deleted", str_oid).to_string(),
                ),
            );
            Ok(map_result)
        });

        let result = mock.delete_series(str_oid).await;

        assert!(result.is_ok());

        let map = result.unwrap();
        assert_eq!(
            map["message"],
            format!("Series with id: '{}' was successfully deleted", str_oid).to_string()
        );
    }

    #[actix_web::test]
    async fn test_delete_series_internal_server_error() {
        let mut mock = MockSeriesRepository::new();
        let oid = ObjectId::new();
        let parsed_oid = oid.to_string();
        let str_oid = parsed_oid.as_str();

        mock.expect_delete_series()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.delete_series(str_oid).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::InternalServerError);
    }

    #[actix_web::test]
    async fn test_delete_series_not_exists() {
        let mut mock = MockSeriesRepository::new();
        let oid = ObjectId::new();
        let parsed_oid = oid.to_string();
        let str_oid = parsed_oid.as_str();

        mock.expect_delete_series()
            .returning(|_| Err(AppError::NotExists));

        let result = mock.delete_series(str_oid).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppError::NotExists);
    }
}
