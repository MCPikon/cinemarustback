use std::{env, str::FromStr};

use crate::{
    error::AppError,
    models::{
        movie::{Movie, MovieResponse},
        review::Review,
        series::Series,
    },
};
use dotenv::dotenv;
use futures_util::{StreamExt, TryStreamExt};
use log::{error, info, warn};
use mongodb::{
    bson::{doc, oid::ObjectId, Regex},
    error::Error,
    options::{CountOptions, FindOptions},
    results::{InsertOneResult, UpdateResult},
    Client, Collection,
};
use serde_json::{Map, Value};

pub struct Database {
    movies: Collection<Movie>,
    series: Collection<Series>,
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
        let db = client.database("cinema-web-db");

        let movies: Collection<Movie> = db.collection("movies");
        let series: Collection<Series> = db.collection("series");
        let reviews: Collection<Review> = db.collection("reviews");

        Database {
            movies,
            series,
            reviews,
        }
    }

    pub async fn create_movie(&self, movie: Movie) -> Result<InsertOneResult, Error> {
        let result = self
            .movies
            .insert_one(&movie, None)
            .await
            .ok()
            .expect(format!("Error creating movie with imdbId: '{}'", movie.imdb_id).as_str());
        Ok(result)
    }

    pub async fn update_movie(&self, movie: Movie) -> Result<UpdateResult, Error> {
        let result = self
            .movies
            .update_one(
                doc! { "_id": movie._id },
                doc! {
                "$set": doc! {
                    "imdbId": &movie.imdb_id,
                    "title": movie.title,
                    "overview": movie.overview,
                    "duration": movie.duration,
                    "releaseDate": movie.release_date,
                    "trailerLink": movie.trailer_link,
                    "genres": movie.genres,
                    "poster": movie.poster,
                    "backdrop": movie.backdrop,
                    "reviewIds": movie.review_ids
                }},
                None,
            )
            .await
            .ok()
            .expect(format!("Error updating movie with imdbId: '{}'", movie.imdb_id).as_str());
        Ok(result)
    }

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
}
