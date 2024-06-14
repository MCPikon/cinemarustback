use std::error::Error;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    #[serde(rename(serialize = "_id", deserialize = "_id"))]
    pub _id: ObjectId,
    pub imdb_id: String,
    pub title: String,
    pub overview: String,
    pub duration: String,
    pub director: String,
    pub release_date: String,
    pub trailer_link: String,
    pub genres: Vec<String>,
    pub poster: String,
    pub backdrop: String,
    pub review_ids: Vec<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MovieDoc {
    #[serde(rename(serialize = "_id", deserialize = "_id"))]
    pub _id: String,
    #[schema(example = "tt12345")]
    pub imdb_id: String,
    #[schema(example = "El lobo de Wall Street")]
    pub title: String,
    #[schema(example = "La nueva película de Martin Scorsese: La biografía de Jordan Belfort.")]
    pub overview: String,
    #[schema(example = "2h 59m")]
    pub duration: String,
    #[schema(example = "Martin Scorsese")]
    pub director: String,
    #[schema(example = "2014-01-17")]
    pub release_date: String,
    #[schema(example = "https://youtu.be/DEMZSa0esCU")]
    pub trailer_link: String,
    #[schema(example = "Crimen, Drama, Comedia")]
    pub genres: Vec<String>,
    #[schema(example = "https://image.tmdb.org/t/p/original/jTlIYjvS16XOpsfvYCTmtEHV10K.jpg")]
    pub poster: String,
    #[schema(example = "https://image.tmdb.org/t/p/original/7Nwnmyzrtd0FkcRyPqmdzTPppQa.jpg")]
    pub backdrop: String,
    pub review_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MovieRequest {
    pub imdb_id: String,
    pub title: String,
    pub overview: String,
    pub duration: String,
    pub director: String,
    pub release_date: String,
    pub trailer_link: String,
    pub genres: Vec<String>,
    pub poster: String,
    pub backdrop: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MovieResponse {
    pub imdb_id: String,
    pub title: String,
    pub duration: String,
    pub release_date: String,
    pub poster: String,
}

impl TryFrom<MovieRequest> for Movie {
    type Error = Box<dyn Error>;

    fn try_from(item: MovieRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: ObjectId::new(),
            imdb_id: item.imdb_id,
            title: item.title,
            overview: item.overview,
            duration: item.duration,
            director: item.director,
            release_date: item.release_date,
            trailer_link: item.trailer_link,
            genres: item.genres,
            poster: item.poster,
            backdrop: item.backdrop,
            review_ids: Vec::new(),
        })
    }
}

impl TryFrom<Movie> for MovieResponse {
    type Error = Box<dyn Error>;

    fn try_from(item: Movie) -> Result<Self, Self::Error> {
        Ok(Self {
            imdb_id: item.imdb_id,
            title: item.title,
            duration: item.duration,
            release_date: item.release_date,
            poster: item.poster,
        })
    }
}
