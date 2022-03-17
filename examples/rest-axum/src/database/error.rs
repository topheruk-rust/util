use crate::model::error::Error as ModelError;

#[derive(Debug)]
pub enum Error {
    NotFound,
    Model(ModelError),
    Mongo(mongodb::error::Error),
    Serialize(bson::ser::Error),
    Deserialize(bson::de::Error),
}

impl From<ModelError> for Error {
    fn from(e: ModelError) -> Self {
        Self::Model(e)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(err: mongodb::error::Error) -> Self {
        Self::Mongo(err)
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
