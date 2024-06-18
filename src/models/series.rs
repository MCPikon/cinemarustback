use std::error::Error;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    title: String,
    release_date: String,
    duration: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    overview: String,
    episode_list: Vec<Episode>,
    poster: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    #[serde(rename(serialize = "_id", deserialize = "_id"))]
    pub _id: ObjectId,
    pub imdb_id: String,
    pub title: String,
    pub overview: String,
    pub number_of_seasons: u32,
    pub creator: String,
    pub release_date: String,
    pub trailer_link: String,
    pub genres: Vec<String>,
    pub season_list: Vec<Season>,
    pub poster: String,
    pub backdrop: String,
    pub review_ids: Vec<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SeriesDoc {
    #[serde(rename(serialize = "_id", deserialize = "_id"))]
    pub _id: String,
    #[schema(example = "tt12345")]
    pub imdb_id: String,
    #[schema(example = "La Casa del Dragón")]
    pub title: String,
    #[schema(example = "Basada en el libro 'Fuego y Sangre' de George R.R. Martin.")]
    pub overview: String,
    #[schema(example = 2)]
    pub number_of_seasons: u32,
    #[schema(example = "George R.R. Martin")]
    pub creator: String,
    #[schema(example = "2021-06-21")]
    pub release_date: String,
    #[schema(example = "https://youtu.be/oBFtJUWuGFI")]
    pub trailer_link: String,
    #[schema(example = "Ciencia Ficción y Fantasía, Drama, Acción y Aventura")]
    pub genres: Vec<String>,
    pub season_list: Vec<Season>,
    #[schema(example = "https://image.tmdb.org/t/p/original/fAos5hPi7TB49KpuIAjvQNZkvwM.jpg")]
    pub poster: String,
    #[schema(example = "https://image.tmdb.org/t/p/original/xtAQ7j9Yd0j4Rjbvx1hW0ENpXjf.jpg")]
    pub backdrop: String,
    pub review_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SeriesRequest {
    pub imdb_id: String,
    pub title: String,
    pub overview: String,
    pub number_of_seasons: u32,
    pub creator: String,
    pub release_date: String,
    pub trailer_link: String,
    pub genres: Vec<String>,
    pub season_list: Vec<Season>,
    pub poster: String,
    pub backdrop: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SeriesResponse {
    #[schema(example = "tt12345")]
    pub imdb_id: String,
    #[schema(example = "La Casa del Dragón")]
    pub title: String,
    #[schema(example = 2)]
    pub number_of_seasons: u32,
    #[schema(example = "2021-06-21")]
    pub release_date: String,
    #[schema(example = "https://image.tmdb.org/t/p/original/fAos5hPi7TB49KpuIAjvQNZkvwM.jpg")]
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
