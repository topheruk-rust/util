pub mod database;
pub mod error;
pub mod handler;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;
