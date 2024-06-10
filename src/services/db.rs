use std::{env, str::FromStr};

use crate::{
    error::AppError,
    models::{
        movie::{Movie, MovieRequest, MovieResponse},
        review::Review,
        series::{Series, SeriesRequest, SeriesResponse},
    },
};
use dotenv::dotenv;
use futures_util::{StreamExt, TryStreamExt};
use log::{error, info, warn};
use mongodb::{
    bson::{doc, oid::ObjectId, to_bson, Regex},
    options::{CountOptions, FindOptions},
    results::{InsertOneResult, UpdateResult},
    Client, Collection,
};
use serde_json::{Map, Value};

pub struct Database {
    movies: Collection<Movie>,
    series: Collection<Series>,
    #[allow(dead_code)] // Provisional hasta que añada los endpoints
    reviews: Collection<Review>,
}

impl Database {
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGO_URI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("An error ocurred trying to connect with MongoDB URI"),
        };

        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("cinema-rust-db");

        let movies: Collection<Movie> = db.collection("movies");
        let series: Collection<Series> = db.collection("series");
        let reviews: Collection<Review> = db.collection("reviews");

        Database {
            movies,
            series,
            reviews,
        }
    }

    // Movies

    pub async fn find_all_movies(
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

    pub async fn find_movie_by_id(&self, id: &str) -> Result<Movie, AppError> {
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

    pub async fn find_movie_by_imdb_id(&self, imdb_id: &str) -> Result<Movie, AppError> {
        info!("GET movies /findByImdbId with id: '{}' executed", imdb_id);
        let re = regex::Regex::new(r"^tt\d+$").unwrap();
        if !re.is_match(imdb_id) {
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

    pub async fn create_movie(&self, movie: Movie) -> Result<InsertOneResult, AppError> {
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
        Ok(result)
    }

    pub async fn delete_movie(&self, id: &str) -> Result<Map<String, Value>, AppError> {
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

    pub async fn movie_exists_by_imdb_id(&self, imdb_id: &str) -> Result<bool, AppError> {
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

    pub async fn series_exists_by_imdb_id(&self, imdb_id: &str) -> Result<bool, AppError> {
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

    pub async fn update_movie(
        &self,
        id: &str,
        movie: MovieRequest,
    ) -> Result<UpdateResult, AppError> {
        info!("UPDATE movies /update with id: '{}' executed", id);
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
        let re = regex::Regex::new(r"^tt\d+$").unwrap();
        if !re.is_match(&movie.imdb_id) {
            error!(
                "Error in movies /update with id: '{}' [{}]",
                id,
                AppError::WrongImdbId.to_string()
            );
            return Err(AppError::WrongImdbId);
        }
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
        Ok(result)
    }

    pub async fn patch_movie(
        &self,
        id: &str,
        field: &str,
        val: &str,
    ) -> Result<UpdateResult, AppError> {
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
            let re = regex::Regex::new(r"^tt\d+$").unwrap();
            if !re.is_match(val) {
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
        Ok(result)
    }

    // Series

    pub async fn find_all_series(
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

    pub async fn find_series_by_id(&self, id: &str) -> Result<Series, AppError> {
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

    pub async fn find_series_by_imdb_id(&self, imdb_id: &str) -> Result<Series, AppError> {
        info!("GET series /findByImdbId with id: '{}' executed", imdb_id);
        let re = regex::Regex::new(r"^tt\d+$").unwrap();
        if !re.is_match(imdb_id) {
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

    pub async fn create_series(&self, series: Series) -> Result<InsertOneResult, AppError> {
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
        Ok(result)
    }

    pub async fn delete_series(&self, id: &str) -> Result<Map<String, Value>, AppError> {
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

    pub async fn update_series(
        &self,
        id: &str,
        series: SeriesRequest,
    ) -> Result<UpdateResult, AppError> {
        info!("UPDATE series /update with id: '{}' executed", id);
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
        let re = regex::Regex::new(r"^tt\d+$").unwrap();
        if !re.is_match(&series.imdb_id) {
            error!(
                "Error in series /update with id: '{}' [{}]",
                id,
                AppError::WrongImdbId.to_string()
            );
            return Err(AppError::WrongImdbId);
        }
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
        Ok(result)
    }

    pub async fn patch_series(
        &self,
        id: &str,
        field: &str,
        val: &str,
    ) -> Result<UpdateResult, AppError> {
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
            let re = regex::Regex::new(r"^tt\d+$").unwrap();
            if !re.is_match(val) {
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
        Ok(result)
    }
}
