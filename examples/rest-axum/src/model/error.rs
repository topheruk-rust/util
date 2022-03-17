#[derive(Debug)]
pub enum Error {
    Serde(serde_json::Error),
    EmptyText,
    BsonSerialize(bson::ser::Error),
    BsonDeserialize(bson::de::Error),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

impl From<bson::ser::Error> for Error {
    fn from(e: bson::ser::Error) -> Self {
        Self::BsonSerialize(e)
    }
}

impl From<bson::de::Error> for Error {
    fn from(e: bson::de::Error) -> Self {
        Self::BsonDeserialize(e)
    }
}
