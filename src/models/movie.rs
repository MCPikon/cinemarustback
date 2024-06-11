use std::error::Error;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    #[serde(rename(serialize = "id", deserialize = "_id"))]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
