use std::fmt;

use chrono::Utc;
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

// You use `serde` to create structs which can serialize & deserialize between BSON:
#[derive(Serialize, Deserialize, Debug)]
pub struct Movie {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub year: i32,
    pub plot: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub released: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct MovieSummary {
    pub title: String,
    pub cast: Vec<String>,
    pub year: i32,
}

impl fmt::Display for MovieSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}",
            self.title,
            self.cast.get(0).unwrap_or(&"- no cast -".to_owned()),
            self.year
        )
    }
}
