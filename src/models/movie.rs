use std::error::Error;

use lazy_static::lazy_static;
use mongodb::bson::oid::ObjectId;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

lazy_static! {
    static ref RE_IMDB_ID: Regex = Regex::new(r"^tt\d+$").unwrap();
    static ref RE_DURATION: Regex = Regex::new(r"^(\d{1,2})h\s(\d{1,2})m$").unwrap();
    static ref RE_DIRECTOR: Regex =
        Regex::new(r"^([a-zA-Z]+\.?)\s([a-zA-Z]+\.?)(?:\s([a-zA-Z]+))?$").unwrap();
    static ref RE_RELEASE_DATE: Regex =
        Regex::new(r"^(\d{4})-([1-9]|0[1-9]|1[0-2])-([1-9]|0[1-9]|[12]\d|3[01])$").unwrap();
    static ref RE_TRAILER_LINK: Regex = Regex::new(r"^((?:https?:)?//)?((?:www|m)\.)?((?:youtube(-nocookie)?\.com|youtu.be))(/(?:[\w\-]+\\?v=|embed/|live/|v/)?)([\w\-]+)(\S+)?$").unwrap();
    static ref RE_REMOTE_IMAGES: Regex = Regex::new(r"(https?://\S+(?:png|jpe?g|webp)\S*)").unwrap();
}

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

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct MovieRequest {
    #[validate(regex(
        path = *RE_IMDB_ID,
        message = "The imdbId must match the following format: 'tt0000'"
    ))]
    pub imdb_id: String,
    #[validate(length(min = 1, message = "The movie title cannot be empty"))]
    pub title: String,
    #[validate(length(min = 1, message = "The film synopsis cannot be empty"))]
    pub overview: String,
    #[validate(regex(
        path = *RE_DURATION,
        message = "The duration must match the following format: '00h 00m'"
    ))]
    pub duration: String,
    #[validate(regex(
        path = *RE_DIRECTOR,
        message = "The director's name must match the following format: 'Name Surname'"
    ))]
    pub director: String,
    #[validate(regex(
        path = *RE_RELEASE_DATE,
        message = "The release date of the movie must match the following format: 'YYYY-MM-DD'"
    ))]
    pub release_date: String,
    #[validate(regex(
        path = *RE_TRAILER_LINK,
        message = "The movie trailer link has to be a valid YouTube URL"
    ))]
    pub trailer_link: String,
    #[validate(custom(
        function = "validate_non_empty_vec",
        message = "The movie has to have at least one genre"
    ))]
    pub genres: Vec<String>,
    #[validate(regex(
        path = *RE_REMOTE_IMAGES,
        message = "The movie poster must be a valid URL with one of these extensions: (.jpg, .jpeg, .png or .webp)"
    ))]
    pub poster: String,
    #[validate(regex(
        path = *RE_REMOTE_IMAGES,
        message = "The movie backdrop image must be a valid URL with one of these extensions: (.jpg, .jpeg, .png or .webp)"
    ))]
    pub backdrop: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MovieResponse {
    #[schema(example = "tt12345")]
    pub imdb_id: String,
    #[schema(example = "El lobo de Wall Street")]
    pub title: String,
    #[schema(example = "2h 59m")]
    pub duration: String,
    #[schema(example = "2014-01-17")]
    pub release_date: String,
    #[schema(example = "https://image.tmdb.org/t/p/original/jTlIYjvS16XOpsfvYCTmtEHV10K.jpg")]
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

fn validate_non_empty_vec(vec: &[String]) -> Result<(), ValidationError> {
    if vec.is_empty() {
        return Err(ValidationError::new("vector_empty"));
    }
    Ok(())
}
