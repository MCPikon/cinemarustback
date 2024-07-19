use std::str::FromStr;

use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use log::{error, info, warn};
use mongodb::{
    bson::{doc, oid::ObjectId, Regex},
    options::{CountOptions, FindOptions},
};
use serde_json::{Map, Value};

use crate::{
    error::AppError,
    models::movie::{Movie, MovieRequest, MovieResponse},
};

use super::{db::Database, series_repo::SeriesRepository};

lazy_static! {
    static ref RE_IMDB_ID: regex::Regex = regex::Regex::new(r"^tt\d+$").unwrap();
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MovieRepository {
    async fn find_all_movies(
        &self,
        title: Option<String>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<Map<String, Value>, AppError>;
    async fn find_movie_by_id(&self, id: &str) -> Result<Movie, AppError>;
    async fn find_movie_by_imdb_id(&self, imdb_id: &str) -> Result<Movie, AppError>;
    async fn create_movie(&self, movie: Movie) -> Result<Map<String, Value>, AppError>;
    async fn delete_movie(&self, id: &str) -> Result<Map<String, Value>, AppError>;
    async fn movie_exists_by_imdb_id(&self, imdb_id: &str) -> Result<bool, AppError>;
    async fn update_movie(
        &self,
        id: &str,
        movie: MovieRequest,
    ) -> Result<Map<String, Value>, AppError>;
    async fn patch_movie(
        &self,
        id: &str,
        field: &str,
        val: &str,
    ) -> Result<Map<String, Value>, AppError>;
}

#[async_trait]
impl MovieRepository for Database {
    async fn find_all_movies(
        &self,
        title: Option<String>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<Map<String, Value>, AppError> {
        info!("GET movies /findAll executed");
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
            .movies
            .count_documents(filter.clone(), CountOptions::default())
            .await
            .ok()
            .expect("Error counting total of movies");
        let total_pages = (total_items as f64 / page_size as f64).ceil() as u64;

        let options = FindOptions::builder()
            .skip((page_num * page_size) as u64)
            .limit(page_size as i64)
            .build();

        let cursor = self
            .movies
            .find(filter, options)
            .await
            .ok()
            .expect("Error finding all movies");

        let movie_list: Vec<MovieResponse> = cursor
            .map(|movie| MovieResponse::try_from(movie.unwrap()))
            .try_collect()
            .await
            .ok()
            .expect("Error collecting movies");

        if movie_list.is_empty() {
            warn!("Warn in movies /findAll [{}]", AppError::Empty.to_string());
            return Err(AppError::Empty);
        }

        result_map.insert(
            "movies".to_string(),
            serde_json::to_value(movie_list).unwrap(),
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

    async fn find_movie_by_id(&self, id: &str) -> Result<Movie, AppError> {
        info!("GET movies /findById with id: '{}' executed", id);
        let obj_id = ObjectId::from_str(id)?;
        let movie: Movie = match self.movies.find_one(doc! {"_id": obj_id}, None).await {
            Ok(Some(movie)) => movie,
            Ok(None) => {
                warn!(
                    "Warn in movies /findById with id: '{}' [{}]",
                    id,
                    AppError::NotFound.to_string()
                );
                return Err(AppError::NotFound);
            }
            Err(_) => {
                error!(
                    "Error in movies /findById with id: '{}' [{}]",
                    id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        Ok(movie)
    }

    async fn find_movie_by_imdb_id(&self, imdb_id: &str) -> Result<Movie, AppError> {
        info!("GET movies /findByImdbId with id: '{}' executed", imdb_id);
        if !RE_IMDB_ID.is_match(imdb_id) {
            error!(
                "Error in movies /findByImdbId with imdbId: '{}' [{}]",
                imdb_id,
                AppError::WrongImdbId.to_string()
            );
            return Err(AppError::WrongImdbId);
        }

        let movie: Movie = match self.movies.find_one(doc! {"imdbId": imdb_id}, None).await {
            Ok(Some(movie)) => movie,
            Ok(None) => {
                warn!(
                    "Warn in movies /findByImdbId with imdbId: '{}' [{}]",
                    imdb_id,
                    AppError::NotFound.to_string()
                );
                return Err(AppError::NotFound);
            }
            Err(_) => {
                error!(
                    "Error in movies /findByImdbId with imdbId: '{}' [{}]",
                    imdb_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        Ok(movie)
    }

    async fn create_movie(&self, movie: Movie) -> Result<Map<String, Value>, AppError> {
        info!("POST movies /new executed");
        if self
            .movie_exists_by_imdb_id(movie.imdb_id.clone().as_str())
            .await?
        {
            warn!(
                "Warn in movies /new [{}]",
                AppError::AlreadyExists.to_string()
            );
            return Err(AppError::AlreadyExists);
        }
        if self
            .series_exists_by_imdb_id(movie.imdb_id.clone().as_str())
            .await?
        {
            warn!(
                "Warn in movies /new [{}]",
                AppError::AlreadyExists.to_string()
            );
            return Err(AppError::AlreadyExists);
        }
        let result = self
            .movies
            .insert_one(&movie, None)
            .await
            .ok()
            .expect(format!("Error creating movie with imdbId: '{}'", movie.imdb_id).as_str());

        let mut map_result: Map<String, Value> = Map::new();
        map_result.insert(
            "message".to_string(),
            Value::String(
                format!(
                    "Movie was successfully created. (id: '{}')",
                    result.inserted_id.as_object_id().unwrap().to_string()
                )
                .to_string(),
            ),
        );
        Ok(map_result)
    }

    async fn delete_movie(&self, id: &str) -> Result<Map<String, Value>, AppError> {
        info!("DELETE movies /delete with id: '{}' executed", id);
        let obj_id = ObjectId::from_str(id)?;
        let del_result = match self.movies.delete_one(doc! {"_id": obj_id}, None).await {
            Ok(res) => res,
            Err(_) => {
                error!(
                    "Error in movies /delete with id: '{}' [{}]",
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
                    format!("Movie with id: '{}' was successfully deleted", id).to_string(),
                ),
            );
        } else {
            warn!(
                "Warn in movies /delete with id: '{}' [{}]",
                obj_id,
                AppError::NotExists.to_string()
            );
            return Err(AppError::NotExists);
        }
        Ok(map_result)
    }

    async fn movie_exists_by_imdb_id(&self, imdb_id: &str) -> Result<bool, AppError> {
        let exists: bool = match self.movies.find_one(doc! { "imdbId": imdb_id }, None).await {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => {
                error!(
                    "Error checking if movie exists with imdbId: '{}' [{}]",
                    imdb_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        Ok(exists)
    }

    async fn update_movie(
        &self,
        id: &str,
        movie: MovieRequest,
    ) -> Result<Map<String, Value>, AppError> {
        info!("PUT movies /update with id: '{}' executed", id);
        let obj_id = ObjectId::from_str(id)?;
        let movie_founded: Movie = match self.movies.find_one(doc! { "_id": obj_id }, None).await {
            Ok(Some(movie)) => movie,
            Ok(None) => {
                warn!(
                    "Warn in movies /update with id: '{}' [{}]",
                    obj_id,
                    AppError::NotExists.to_string()
                );
                return Err(AppError::NotExists);
            }
            Err(_) => {
                error!(
                    "Error in movies /update with id: '{}' [{}]",
                    obj_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        let exists_imdb_id_movie: bool = self.movie_exists_by_imdb_id(&movie.imdb_id).await?;
        let exists_imdb_id_series: bool = self.series_exists_by_imdb_id(&movie.imdb_id).await?;
        if (exists_imdb_id_movie || exists_imdb_id_series) && movie_founded.imdb_id != movie.imdb_id
        {
            error!(
                "Error in movies /update with id: '{}' [{}]",
                obj_id,
                AppError::ImdbIdInUse.to_string()
            );
            return Err(AppError::ImdbIdInUse);
        }
        let result = self
            .movies
            .update_one(
                doc! { "_id": obj_id },
                doc! {
                "$set": doc! {
                    "imdbId": movie.imdb_id,
                    "title": movie.title,
                    "overview": movie.overview,
                    "duration": movie.duration,
                    "releaseDate": movie.release_date,
                    "trailerLink": movie.trailer_link,
                    "genres": movie.genres,
                    "poster": movie.poster,
                    "backdrop": movie.backdrop
                }},
                None,
            )
            .await
            .ok()
            .expect(format!("Error updating movie with id: '{}'", id).as_str());
        let mut map_result: Map<String, Value> = Map::new();
        map_result.insert(
            "message".to_string(),
            Value::String(if result.modified_count != 0 {
                format!("Movie with id: '{}' was successfully updated", id)
            } else {
                "Fields have the same value, no update was performed".to_string()
            }),
        );
        Ok(map_result)
    }

    async fn patch_movie(
        &self,
        id: &str,
        field: &str,
        val: &str,
    ) -> Result<Map<String, Value>, AppError> {
        info!("PATCH movies /patch with id: '{}' executed", id);
        let fields_vec: Vec<&str> = vec![
            "imdbId",
            "title",
            "overview",
            "duration",
            "director",
            "releaseDate",
            "trailerLink",
            "genres",
            "poster",
            "backdrop",
        ];
        let obj_id = ObjectId::from_str(id)?;
        if !fields_vec.contains(&field) {
            warn!(
                "Warn in movies /patch with id: '{}' [{}]",
                obj_id,
                AppError::FieldNotAllowed.to_string()
            );
            return Err(AppError::FieldNotAllowed);
        }
        let movie_founded: Movie = match self.movies.find_one(doc! { "_id": obj_id }, None).await {
            Ok(Some(movie)) => movie,
            Ok(None) => {
                warn!(
                    "Warn in movies /patch with id: '{}' [{}]",
                    obj_id,
                    AppError::NotExists.to_string()
                );
                return Err(AppError::NotExists);
            }
            Err(_) => {
                error!(
                    "Error in movies /patch with id: '{}' [{}]",
                    obj_id,
                    AppError::InternalServerError.to_string()
                );
                return Err(AppError::InternalServerError);
            }
        };
        if field == "imdbId" {
            if !RE_IMDB_ID.is_match(val) {
                error!(
                    "Error in movies /patch with id: '{}' [{}]",
                    id,
                    AppError::WrongImdbId.to_string()
                );
                return Err(AppError::WrongImdbId);
            }
            let exists_imdb_id_movie: bool = self.movie_exists_by_imdb_id(val).await?;
            let exists_imdb_id_series: bool = self.series_exists_by_imdb_id(val).await?;
            if (exists_imdb_id_movie || exists_imdb_id_series) && movie_founded.imdb_id != val {
                error!(
                    "Error in movies /patch with id: '{}' [{}]",
                    obj_id,
                    AppError::ImdbIdInUse.to_string()
                );
                return Err(AppError::ImdbIdInUse);
            }
        }
        let result = self
            .movies
            .update_one(
                doc! { "_id": obj_id },
                doc! {
                "$set": doc! {
                    field: val
                }},
                None,
            )
            .await
            .ok()
            .expect(format!("Error patching movie with id: '{}'", id).as_str());
        let mut map_result: Map<String, Value> = Map::new();
        map_result.insert(
            "message".to_string(),
            Value::String(if result.modified_count != 0 {
                format!("Movie {} with id: '{}' was successfully patched", field, id)
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

    // Auxiliar Functions

    fn build_movie_mock(oid: ObjectId) -> Movie {
        Movie {
            _id: oid,
            imdb_id: "tt12345".to_string(),
            title: "El lobo de Wall Street".to_string(),
            director: "Martin Scorsese".to_string(),
            overview: "Testing movies...".to_string(),
            release_date: "2002-12-4".to_string(),
            duration: "2h 54m".to_string(),
            trailer_link: "https://youtube.com/dasDsdXsDS".to_string(),
            genres: vec![
                "Crimen".to_string(),
                "Drama".to_string(),
                "Ciencia Ficción".to_string(),
            ],
            poster: "https://moviedb.com/lobo/lobo_poster.jpg".to_string(),
            backdrop: "https://moviedb.com/lobo/lobo_backdrop.jpg".to_string(),
            review_ids: vec![ObjectId::new()],
        }
    }

    fn build_movie_req_mock() -> MovieRequest {
        MovieRequest {
            imdb_id: "tt12345".to_string(),
            title: "Casino".to_string(),
            overview: "Película que trata de la mafia de los casinos de Las Vegas".to_string(),
            director: "Martin Scorsese".to_string(),
            duration: "2h 54m".to_string(),
            release_date: "1990-3-4".to_string(),
            genres: vec!["Crímen".to_string(), "Drama".to_string()],
            trailer_link: "https://youtube.com/video/ds1281o3l1h".to_string(),
            poster: "https://moviedb.com/casino/poster.jpg".to_string(),
            backdrop: "https://moviedb.com/casino/poster.jpg".to_string(),
        }
    }

    // Unit Tests

    #[actix_web::test]
    async fn test_find_all_movies_ok() {
        let mut mock = MockMovieRepository::new();

        mock.expect_find_all_movies().returning(|_, _, _| {
            let mut result_map = serde_json::Map::new();
            let movie = MovieResponse {
                imdb_id: "tt12345".to_string(),
                title: "Casino".to_string(),
                duration: "2h 54m".to_string(),
                release_date: "1990-3-4".to_string(),
                poster: "https://moviedb.com/casino/poster.jpg".to_string(),
            };
            result_map.insert(
                "movies".to_string(),
                serde_json::to_value(vec![movie]).unwrap(),
            );
            result_map.insert("currentPage".to_string(), serde_json::to_value(1).unwrap());
            result_map.insert("totalItems".to_string(), serde_json::to_value(1).unwrap());
            result_map.insert("totalPages".to_string(), serde_json::to_value(1).unwrap());
            Ok(result_map)
        });

        let result = mock
            .find_all_movies(Some("Casino".to_string()), Some(1), Some(10))
            .await;

        let map = result.unwrap();
        assert_eq!(map.get("currentPage").unwrap(), 1);
        assert_eq!(map.get("totalItems").unwrap(), 1);
        assert_eq!(map.get("totalPages").unwrap(), 1);

        let movie_list = map.get("movies").unwrap().as_array().unwrap();
        assert_eq!(movie_list.len(), 1);
        assert_eq!(movie_list[0].get("title").unwrap(), "Casino");
    }

    #[actix_web::test]
    async fn test_find_all_movies_empty_list() {
        let mut mock = MockMovieRepository::new();

        mock.expect_find_all_movies()
            .returning(|_, _, _| Err(AppError::Empty));

        let result = mock.find_all_movies(None, Some(1), Some(10)).await;
        assert!(result.is_err_and(|err| err == AppError::Empty));
    }

    #[actix_web::test]
    async fn test_find_all_movies_internal_server_error() {
        let mut mock = MockMovieRepository::new();

        mock.expect_find_all_movies()
            .returning(|_, _, _| Err(AppError::InternalServerError));

        let result = mock.find_all_movies(None, Some(1), Some(10)).await;
        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_find_movie_by_id_ok() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_movie_by_id()
            .returning(move |_| Ok(build_movie_mock(oid)));

        let result = mock.find_movie_by_id(oid.to_string().as_str()).await;
        assert!(result.is_ok());

        let movie = result.unwrap();
        assert_eq!(movie.imdb_id, "tt12345".to_string());
        assert_eq!(movie.title, "El lobo de Wall Street".to_string());
    }

    #[actix_web::test]
    async fn test_find_movie_by_id_cannot_parse_obj_id() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_movie_by_id()
            .returning(|_| Err(AppError::CannotParseObjId));

        let result = mock.find_movie_by_id(oid.to_string().as_str()).await;
        assert!(result.is_err_and(|err| err == AppError::CannotParseObjId));
    }

    #[actix_web::test]
    async fn test_find_movie_by_id_not_found() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_movie_by_id()
            .returning(|_| Err(AppError::NotFound));

        let result = mock.find_movie_by_id(oid.to_string().as_str()).await;
        assert!(result.is_err_and(|err| err == AppError::NotFound));
    }

    #[actix_web::test]
    async fn test_find_movie_by_id_internal_server_error() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();

        mock.expect_find_movie_by_id()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.find_movie_by_id(oid.to_string().as_str()).await;
        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_find_movie_by_imdb_id_ok() {
        let mut mock = MockMovieRepository::new();

        mock.expect_find_movie_by_imdb_id()
            .returning(|_| Ok(build_movie_mock(ObjectId::new())));

        let result = mock.find_movie_by_imdb_id("tt12345").await;
        assert!(result.is_ok());

        let movie = result.unwrap();
        assert_eq!(movie.imdb_id, "tt12345".to_string());
        assert_eq!(movie.title, "El lobo de Wall Street".to_string());
    }

    #[actix_web::test]
    async fn test_find_movie_by_imdb_id_wrong_imdb_id() {
        let mut mock = MockMovieRepository::new();

        mock.expect_find_movie_by_imdb_id()
            .returning(|_| Err(AppError::WrongImdbId));

        let result = mock.find_movie_by_imdb_id("tfd2312").await;
        assert!(result.is_err_and(|err| err == AppError::WrongImdbId));
    }

    #[actix_web::test]
    async fn test_find_movie_by_imdb_id_not_found() {
        let mut mock = MockMovieRepository::new();

        mock.expect_find_movie_by_imdb_id()
            .returning(|_| Err(AppError::NotFound));

        let result = mock.find_movie_by_imdb_id("tt54321").await;
        assert!(result.is_err_and(|err| err == AppError::NotFound));
    }

    #[actix_web::test]
    async fn test_find_movie_by_imdb_id_internal_server_error() {
        let mut mock = MockMovieRepository::new();

        mock.expect_find_movie_by_imdb_id()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.find_movie_by_imdb_id("tt54231").await;
        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_create_movie_ok() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let movie = build_movie_mock(oid);
        let crt_msg = format!("Movie was successfully created. (id: '{}')", oid);

        mock.expect_create_movie().returning({
            let msg = crt_msg.clone();
            move |_| {
                let mut map_result: Map<String, Value> = Map::new();
                map_result.insert("message".to_string(), Value::String(msg.clone()));
                Ok(map_result)
            }
        });

        let result = mock.create_movie(movie).await;
        assert!(result.is_ok_and(|map| map["message"] == crt_msg));
    }

    #[actix_web::test]
    async fn test_create_movie_already_exists() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let movie = build_movie_mock(oid);

        mock.expect_create_movie()
            .returning(|_| Err(AppError::AlreadyExists));

        let result = mock.create_movie(movie).await;
        assert!(result.is_err_and(|err| err == AppError::AlreadyExists));
    }

    #[actix_web::test]
    async fn test_create_movie_internal_server_error() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let movie = build_movie_mock(oid);

        mock.expect_create_movie()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.create_movie(movie).await;
        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_delete_movie_ok() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let del_msg = format!("Movie with id: '{}' was successfully deleted", oid);

        mock.expect_delete_movie().returning({
            let msg = del_msg.clone();
            move |_| {
                let mut map_result: Map<String, Value> = Map::new();
                map_result.insert("message".to_string(), Value::String(msg.clone()));
                Ok(map_result)
            }
        });

        let result = mock.delete_movie(oid.to_string().as_str()).await;
        assert!(result.is_ok_and(|map| map["message"] == del_msg));
    }

    #[actix_web::test]
    async fn test_delete_movie_internal_server_error() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();

        mock.expect_delete_movie()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.delete_movie(oid.to_string().as_str()).await;
        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_delete_movie_not_exists() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();

        mock.expect_delete_movie()
            .returning(|_| Err(AppError::NotExists));

        let result = mock.delete_movie(oid.to_string().as_str()).await;

        assert!(result.is_err_and(|err| err == AppError::NotExists));
    }

    #[actix_web::test]
    async fn test_movie_exists_by_imdb_id_true() {
        let mut mock = MockMovieRepository::new();

        mock.expect_movie_exists_by_imdb_id()
            .returning(|_| Ok(true));

        let result = mock.movie_exists_by_imdb_id("tt12345").await;
        assert!(result.is_ok_and(|exists| exists));
    }

    #[actix_web::test]
    async fn test_movie_exists_by_imdb_id_false() {
        let mut mock = MockMovieRepository::new();

        mock.expect_movie_exists_by_imdb_id()
            .returning(|_| Ok(false));

        let result = mock.movie_exists_by_imdb_id("tt54321").await;
        assert!(result.is_ok_and(|exists| !exists));
    }

    #[actix_web::test]
    async fn test_movie_exists_by_imdb_id_internal_server_error() {
        let mut mock = MockMovieRepository::new();

        mock.expect_movie_exists_by_imdb_id()
            .returning(|_| Err(AppError::InternalServerError));

        let result = mock.movie_exists_by_imdb_id("tt54321").await;
        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_update_movie_ok() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let movie = build_movie_req_mock();
        let upt_msg = format!("Movie with id: '{}' was successfully updated", oid);

        mock.expect_update_movie().returning({
            let msg = upt_msg.clone();
            move |_, _| {
                let mut map_result: Map<String, Value> = Map::new();
                map_result.insert("message".to_string(), Value::String(msg.clone()));
                Ok(map_result)
            }
        });

        let result = mock.update_movie(oid.to_string().as_str(), movie).await;
        assert!(result.is_ok_and(|map| map["message"] == upt_msg));
    }

    #[actix_web::test]
    async fn test_update_movie_not_exists() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let movie = build_movie_req_mock();

        mock.expect_update_movie()
            .returning(|_, _| Err(AppError::NotExists));

        let result = mock.update_movie(oid.to_string().as_str(), movie).await;
        assert!(result.is_err_and(|err| err == AppError::NotExists));
    }

    #[actix_web::test]
    async fn test_update_movie_internal_server_error() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let movie = build_movie_req_mock();

        mock.expect_update_movie()
            .returning(|_, _| Err(AppError::InternalServerError));

        let result = mock.update_movie(oid.to_string().as_str(), movie).await;
        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }

    #[actix_web::test]
    async fn test_update_movie_imdb_id_in_use() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let movie = build_movie_req_mock();

        mock.expect_update_movie()
            .returning(|_, _| Err(AppError::ImdbIdInUse));

        let result = mock.update_movie(oid.to_string().as_str(), movie).await;
        assert!(result.is_err_and(|err| err == AppError::ImdbIdInUse));
    }

    // TODO: refactorizar de aquí para abajo (si no te acuerdas mira los repos de series o review)

    #[actix_web::test]
    async fn test_patch_movie_ok() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let field = "title";
        let val_mock = "El Cabo del Miedo";
        let pt_msg = format!(
            "Movie {} with id: '{}' was successfully patched",
            field, oid
        );

        mock.expect_patch_movie().returning({
            let msg = pt_msg.clone();
            move |_, _, _| {
                let mut map_result: Map<String, Value> = Map::new();
                map_result.insert("message".to_string(), Value::String(msg.clone()));
                Ok(map_result)
            }
        });

        let result = mock
            .patch_movie(oid.to_string().as_str(), field, val_mock)
            .await;

        assert!(result.is_ok_and(|map| map["message"] == pt_msg));
    }

    #[actix_web::test]
    async fn test_patch_movie_cannot_parse_object_id() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let field = "title";
        let val_mock = "El Cabo del Miedo";

        mock.expect_patch_movie()
            .returning(|_, _, _| Err(AppError::CannotParseObjId));

        let result = mock
            .patch_movie(oid.to_string().as_str(), field, val_mock)
            .await;

        assert!(result.is_err_and(|err| err == AppError::CannotParseObjId));
    }

    #[actix_web::test]
    async fn test_patch_movie_field_not_allowed() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let field = "titleeee";
        let val_mock = "El Cabo del Miedo";

        mock.expect_patch_movie()
            .returning(|_, _, _| Err(AppError::FieldNotAllowed));

        let result = mock
            .patch_movie(oid.to_string().as_str(), field, val_mock)
            .await;

        assert!(result.is_err_and(|err| err == AppError::FieldNotAllowed));
    }

    #[actix_web::test]
    async fn test_patch_movie_wrong_imdb_id() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let field = "imdbId";
        let val_mock = "tF123asS";

        mock.expect_patch_movie()
            .returning(|_, _, _| Err(AppError::WrongImdbId));

        let result = mock
            .patch_movie(oid.to_string().as_str(), field, val_mock)
            .await;

        assert!(result.is_err_and(|err| err == AppError::WrongImdbId));
    }

    #[actix_web::test]
    async fn test_patch_movie_imdb_id_in_use() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let field = "imdbId";
        let val_mock = "tt12345";

        mock.expect_patch_movie()
            .returning(|_, _, _| Err(AppError::ImdbIdInUse));

        let result = mock
            .patch_movie(oid.to_string().as_str(), field, val_mock)
            .await;

        assert!(result.is_err_and(|err| err == AppError::ImdbIdInUse));
    }

    #[actix_web::test]
    async fn test_patch_movie_internal_server_error() {
        let mut mock = MockMovieRepository::new();
        let oid = ObjectId::new();
        let field = "imdbId";
        let val_mock = "tt12345";

        mock.expect_patch_movie()
            .returning(|_, _, _| Err(AppError::InternalServerError));

        let result = mock
            .patch_movie(oid.to_string().as_str(), field, val_mock)
            .await;

        assert!(result.is_err_and(|err| err == AppError::InternalServerError));
    }
}
