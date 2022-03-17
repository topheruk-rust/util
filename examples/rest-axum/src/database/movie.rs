use bson::{oid::ObjectId, Document};
use futures::TryStreamExt;
use mongodb::Collection;

use crate::model::movie::Movie;

use super::Result;

#[derive(Debug, Clone)]
pub struct MovieRepo {
    db: Collection<Movie>,
}

impl From<mongodb::Client> for MovieRepo {
    fn from(cli: mongodb::Client) -> Self {
        Self {
            db: cli.database("sample_mflix").collection("movies"),
        }
    }
}

impl MovieRepo {
    pub async fn find(&self, pipeline: impl IntoIterator<Item = Document>) -> Result<Vec<Movie>> {
        // TODO: check why this fails when using find<Type>

        let mut cur = self.db.aggregate(pipeline, None).await?;

        let mut result = vec![];
        while let Some(doc) = cur.try_next().await? {
            let movie: Movie = bson::from_document(doc)?;
            result.push(movie)
        }

        Ok(result)
    }

    pub async fn find_one(&self, doc: Document) -> Result<Option<Movie>> {
        let movie = self.db.find_one(Some(doc), None).await?;

        Ok(movie)
    }

    pub async fn insert_one(&mut self, movie: Movie) -> Result<Option<ObjectId>> {
        let result = self.db.insert_one(movie, None).await?;
        let id = result.inserted_id.as_object_id();

        Ok(id)
    }

    // pub async fn delete(&mut self, id: Uuid) -> Result<()>;
    // pub async fn update(&mut self, id: Uuid, dto: U) -> Result<T>;
}
