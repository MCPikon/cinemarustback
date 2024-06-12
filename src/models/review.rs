use std::error::Error;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewRequest {
    pub title: String,
    pub rating: u32,
    pub body: String,
    pub imdb_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewUpdate {
    pub title: String,
    pub rating: u32,
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
