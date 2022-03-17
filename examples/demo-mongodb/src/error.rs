#[derive(Clone, Debug)]
pub enum Error {
    Database(mongodb::error::Error),
    Serialize(bson::ser::Error),
    Deserialize(bson::de::Error),
}

impl From<mongodb::error::Error> for Error {
    fn from(err: mongodb::error::Error) -> Self {
        Self::Database(err)
    }
}

impl From<bson::ser::Error> for Error {
    fn from(e: bson::ser::Error) -> Self {
        Self::Serialize(e)
    }
}

impl From<bson::de::Error> for Error {
    fn from(e: bson::de::Error) -> Self {
        Self::Deserialize(e)
    }
}

use std::fmt::Display;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}
