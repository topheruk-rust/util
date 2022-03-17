use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::database::error::Error as DbError;
use crate::model::error::Error as ModelError;

#[derive(Debug)]
pub enum Error {
    Database(DbError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<DbError> for Error {
    fn from(e: DbError) -> Self {
        Self::Database(e)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (code, message) = match self {
            // TODO: find a better way of handling this
            Error::Database(DbError::NotFound) => (StatusCode::NOT_FOUND, "not found".to_string()),
            Error::Database(DbError::Mongo(e)) => (StatusCode::NOT_FOUND, e.to_string()),
            Error::Database(DbError::Model(ModelError::EmptyText)) => {
                (StatusCode::BAD_REQUEST, "text field is empty".to_string())
            }
            Error::Database(DbError::Model(ModelError::BsonSerialize(e))) => {
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Error::Database(DbError::Model(ModelError::BsonDeserialize(e))) => {
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Error::Database(DbError::Model(ModelError::Serde(e))) => {
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Error::Database(DbError::Deserialize(e)) => (StatusCode::BAD_REQUEST, e.to_string()),
            Error::Database(DbError::Serialize(e)) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        let body = Json(json!({ "error": message }));

        (code, body).into_response()
    }
}
