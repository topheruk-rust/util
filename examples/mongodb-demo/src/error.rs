use std::{
    error,
    fmt::{self, Display},
};

#[derive(Clone, Debug)]
pub enum Error {
    MongoDbError(mongodb::error::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // I think the type for the `ref` object should be `mongodb::error::Error` but I could be wrong
            Error::MongoDbError(ref e) => write!(f, "{}", e),
        }
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(err: mongodb::error::Error) -> Self {
        Self::MongoDbError(err)
    }
}

impl error::Error for Error {}
