use std::error::Error;

use lazy_static::lazy_static;
use mongodb::bson::oid::ObjectId;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

lazy_static! {
    static ref RE_IMDB_ID: Regex = Regex::new(r"^tt\d+$").unwrap();
    static ref RE_CREATOR: Regex =
        Regex::new(r"^([a-zA-Z]+\.?)\s([a-zA-Z]+\.?)(?:\s([a-zA-Z]+))?$").unwrap();
    static ref RE_RELEASE_DATE: Regex =
        Regex::new(r"^(\d{4})-([1-9]|0[1-9]|1[0-2])-([1-9]|0[1-9]|[12]\d|3[01])$").unwrap();
    static ref RE_TRAILER_LINK: Regex = Regex::new(r"^((?:https?:)?//)?((?:www|m)\.)?((?:youtube(-nocookie)?\.com|youtu.be))(/(?:[\w\-]+\\?v=|embed/|live/|v/)?)([\w\-]+)(\S+)?$").unwrap();
    static ref RE_REMOTE_IMAGES: Regex = Regex::new(r"(https?://\S+(?:png|jpe?g|webp)\S*)").unwrap();
    static ref RE_DURATION: Regex = Regex::new(r"^(?:(\d{1,2})h(?: (\d{1,2})m)?|(\d{1,2})m)$").unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    #[validate(length(min = 1, message = "The episode title cannot be empty"))]
    title: String,
    #[validate(regex(
        path = *RE_RELEASE_DATE,
        message = "The release date of the episode must match the following format: 'YYYY-MM-DD'"
    ))]
    release_date: String,
    #[validate(regex(
        path = *RE_DURATION,
        message = "The duration must match the following formats: '00h 00m', '00h' or '00m'"
    ))]
    duration: String,
    #[validate(length(min = 1, message = "The episode description cannot be empty"))]
    description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    #[validate(length(min = 1, message = "The season overview cannot be empty"))]
    overview: String,
    #[validate(nested)]
    #[validate(custom(
        function = "validate_non_empty_vec",
        message = "The season has to have at least one episode"
    ))]
    episode_list: Vec<Episode>,
    #[validate(regex(
        path = *RE_REMOTE_IMAGES,
        message = "The series poster must be a valid URL with one of these extensions: (.jpg, .jpeg, .png or .webp)"
    ))]
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

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SeriesRequest {
    #[validate(regex(
        path = *RE_IMDB_ID,
        message = "The imdbId must match the following format: 'tt0000'"
    ))]
    pub imdb_id: String,
    #[validate(length(min = 1, message = "The series title cannot be empty"))]
    pub title: String,
    #[validate(length(min = 1, message = "The series overview cannot be empty"))]
    pub overview: String,
    #[validate(range(min = 0, message = "Number of season of series must be more than 0"))]
    pub number_of_seasons: u32,
    #[validate(regex(
        path = *RE_CREATOR,
        message = "The creator's name must match the following format: 'Name Surname'"
    ))]
    pub creator: String,
    #[validate(regex(
        path = *RE_RELEASE_DATE,
        message = "The release date of the series must match the following format: 'YYYY-MM-DD'"
    ))]
    pub release_date: String,
    #[validate(regex(
        path = *RE_TRAILER_LINK,
        message = "The series trailer link has to be a valid YouTube URL"
    ))]
    pub trailer_link: String,
    #[validate(custom(
        function = "validate_non_empty_vec",
        message = "The series has to have at least one genre"
    ))]
    pub genres: Vec<String>,
    #[validate(nested)]
    #[validate(custom(
        function = "validate_non_empty_vec",
        message = "The series has to have at least one season"
    ))]
    pub season_list: Vec<Season>,
    #[validate(regex(
        path = *RE_REMOTE_IMAGES,
        message = "The series poster must be a valid URL with one of these extensions: (.jpg, .jpeg, .png or .webp)"
    ))]
    pub poster: String,
    #[validate(regex(
        path = *RE_REMOTE_IMAGES,
        message = "The series backdrop image must be a valid URL with one of these extensions: (.jpg, .jpeg, .png or .webp)"
    ))]
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

fn validate_non_empty_vec<T>(vec: &[T]) -> Result<(), ValidationError> {
    if vec.is_empty() {
        return Err(ValidationError::new("vector_empty"));
    }
    Ok(())
}
