use std::error::Error;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Movie {
    #[serde(rename(serialize = "id", deserialize = "_id"))]
    pub _id: ObjectId,
    #[serde(rename = "imdbId")]
    pub imdb_id: String,
    pub title: String,
    pub overview: String,
    pub duration: String,
    pub director: String,
    #[serde(rename = "releaseDate")]
    pub release_date: String,
    #[serde(rename = "trailerLink")]
    pub trailer_link: String,
    pub genres: Vec<String>,
    pub poster: String,
    pub backdrop: String,
    #[serde(rename = "reviewIds")]
    pub review_ids: Vec<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieRequest {
    #[serde(rename = "imdbId")]
    pub imdb_id: String,
    pub title: String,
    pub overview: String,
    pub duration: String,
    pub director: String,
    #[serde(rename = "releaseDate")]
    pub release_date: String,
    #[serde(rename = "trailerLink")]
    pub trailer_link: String,
    pub genres: Vec<String>,
    pub poster: String,
    pub backdrop: String,
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
