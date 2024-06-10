use std::error::Error;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Episode {
    title: String,
    #[serde(rename = "releaseDate")]
    release_date: String,
    duration: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Season {
    overview: String,
    #[serde(rename = "episodeList")]
    episode_list: Vec<Episode>,
    poster: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Series {
    #[serde(rename(serialize = "id", deserialize = "_id"))]
    pub _id: ObjectId,
    #[serde(rename = "imdbId")]
    pub imdb_id: String,
    pub title: String,
    pub overview: String,
    #[serde(rename = "numberOfSeasons")]
    pub number_of_seasons: u32,
    pub creator: String,
    #[serde(rename = "releaseDate")]
    pub release_date: String,
    #[serde(rename = "trailerLink")]
    pub trailer_link: String,
    pub genres: Vec<String>,
    #[serde(rename = "seasonList")]
    pub season_list: Vec<Season>,
    pub poster: String,
    pub backdrop: String,
    #[serde(rename = "reviewIds")]
    pub review_ids: Vec<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesRequest {
    #[serde(rename = "imdbId")]
    pub imdb_id: String,
    pub title: String,
    pub overview: String,
    #[serde(rename = "numberOfSeasons")]
    pub number_of_seasons: u32,
    pub creator: String,
    #[serde(rename = "releaseDate")]
    pub release_date: String,
    #[serde(rename = "trailerLink")]
    pub trailer_link: String,
    pub genres: Vec<String>,
    #[serde(rename = "seasonList")]
    pub season_list: Vec<Season>,
    pub poster: String,
    pub backdrop: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesResponse {
    #[serde(rename = "imdbId")]
    pub imdb_id: String,
    pub title: String,
    #[serde(rename = "numberOfSeasons")]
    pub number_of_seasons: u32,
    #[serde(rename = "releaseDate")]
    pub release_date: String,
    pub poster: String,
}

impl TryFrom<SeriesRequest> for Series {
    type Error = Box<dyn Error>;

    fn try_from(item: SeriesRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: ObjectId::new(),
            imdb_id: item.imdb_id,
            title: item.title,
            overview: item.overview,
            number_of_seasons: item.number_of_seasons,
            creator: item.creator,
            release_date: item.release_date,
            trailer_link: item.trailer_link,
            genres: item.genres,
            season_list: item.season_list,
            poster: item.poster,
            backdrop: item.backdrop,
            review_ids: Vec::new(),
        })
    }
}

impl TryFrom<Series> for SeriesResponse {
    type Error = Box<dyn Error>;

    fn try_from(item: Series) -> Result<Self, Self::Error> {
        Ok(Self {
            imdb_id: item.imdb_id,
            title: item.title,
            number_of_seasons: item.number_of_seasons,
            release_date: item.release_date,
            poster: item.poster,
        })
    }
}
