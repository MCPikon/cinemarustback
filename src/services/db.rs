use std::env;

use crate::models::{movie::Movie, review::Review, series::Series};
use dotenv::dotenv;
use mongodb::{Client, Collection};

pub struct Database {
    pub movies: Collection<Movie>,
    pub series: Collection<Series>,
    pub reviews: Collection<Review>,
}

impl Database {
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGO_URI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("An error ocurred trying to connect with MongoDB URI"),
        };

        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("cinema-rust-db");

        Database {
            movies: db.collection("movies"),
            series: db.collection("series"),
            reviews: db.collection("reviews"),
        }
    }
}
