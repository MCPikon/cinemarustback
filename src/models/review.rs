use std::error::Error;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Review {
    pub _id: ObjectId,
    pub title: String,
    pub rating: u32,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewRequest {
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
