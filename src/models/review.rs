use std::error::Error;

use lazy_static::lazy_static;
use mongodb::bson::{oid::ObjectId, DateTime};
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

lazy_static! {
    static ref RE_IMDB_ID: Regex = Regex::new(r"^tt\d+$").unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    #[serde(rename(serialize = "_id", deserialize = "_id"))]
    pub _id: ObjectId,
    pub title: String,
    pub rating: u32,
    pub body: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResponse {
    #[serde(rename(serialize = "_id", deserialize = "_id"))]
    pub _id: ObjectId,
    pub title: String,
    pub rating: u32,
    pub body: String,
    #[serde(with = "iso_date_format")]
    pub created_at: DateTime,
    #[serde(with = "iso_date_format")]
    pub updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResponseDoc {
    #[serde(rename(serialize = "_id", deserialize = "_id"))]
    pub _id: String,
    #[schema(example = "Una secuela muy a la altura de la anterior.")]
    pub title: String,
    #[schema(example = 4)]
    pub rating: u32,
    #[schema(example = "La verdad que nos quedamos con ganas de más en esta película.")]
    pub body: String,
    #[schema(value_type = String, format = DateTime, example = "2024-05-07T11:56:05.792+00:00")]
    pub created_at: String,
    #[schema(value_type = String, format = DateTime, example = "2024-05-07T11:56:05.792+00:00")]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ReviewRequest {
    #[validate(length(min = 1, message = "The review title cannot be empty"))]
    pub title: String,
    #[validate(range(min = 0, max = 5, message = "The rating must be between 0 and 5"))]
    pub rating: u32,
    #[validate(length(min = 1, message = "The review body cannot be empty"))]
    pub body: String,
    #[validate(regex(
        path = *RE_IMDB_ID,
        message = "The imdbId must match the following format: 'tt0000'"
    ))]
    pub imdb_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct ReviewUpdate {
    #[validate(length(min = 1, message = "The review title cannot be empty"))]
    pub title: String,
    #[validate(range(min = 0, max = 5, message = "The rating must be between 0 and 5"))]
    pub rating: u32,
    #[validate(length(min = 1, message = "The review body cannot be empty"))]
    pub body: String,
}

impl TryFrom<ReviewRequest> for Review {
    type Error = Box<dyn Error>;

    fn try_from(item: ReviewRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: ObjectId::new(),
            title: item.title,
            rating: item.rating,
            body: item.body,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        })
    }
}

impl TryFrom<ReviewUpdate> for Review {
    type Error = Box<dyn Error>;

    fn try_from(item: ReviewUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: ObjectId::new(),
            title: item.title,
            rating: item.rating,
            body: item.body,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        })
    }
}

impl TryFrom<Review> for ReviewResponse {
    type Error = Box<dyn Error>;

    fn try_from(item: Review) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: item._id,
            title: item.title,
            rating: item.rating,
            body: item.body,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
    }
}

// Ser/De for ReviewResponse model datetime fields
mod iso_date_format {
    use chrono::{DateTime, Utc};
    use mongodb::bson::DateTime as BsonDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &BsonDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let datetime: DateTime<Utc> = date.clone().into();
        let formatted_date = datetime.to_rfc3339();
        serializer.serialize_str(&formatted_date)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BsonDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let formatted_date = String::deserialize(deserializer)?;
        let datetime =
            DateTime::parse_from_rfc3339(&formatted_date).map_err(serde::de::Error::custom)?;
        Ok(BsonDateTime::from_chrono(datetime.with_timezone(&Utc)))
    }
}
