use bson::{doc, oid::ObjectId, Bson, Document};
use futures::StreamExt;
use mongodb::Collection;

use crate::{
    model::{Movie, MovieSummary},
    result::Result,
};

pub struct Repo {
    movies: Collection<Document>,
}

impl From<mongodb::Client> for Repo {
    fn from(cli: mongodb::Client) -> Self {
        Self {
            movies: cli.database("sample_mflix").collection("movies"),
        }
    }
}

impl Repo {
    pub async fn find_movie(&self, result_id: ObjectId) -> Result<Movie> {
        let movie = self
            .movies
            .find_one(Some(doc! { "_id":  result_id }), None)
            .await?
            .expect("Document not found");

        Ok(bson::from_bson::<Movie>(Bson::Document(movie))?)
    }

    pub async fn insert_movie(&self, movie: &Movie) -> Result<ObjectId> {
        let value = bson::to_bson(movie)?;
        let document = value.as_document().ok_or(0).unwrap(); // FIXME: safe to unwrap

        let insert_result = self.movies.insert_one(document.clone(), None).await?;
        let result_id = insert_result.inserted_id.as_object_id().unwrap(); // FIXME: don't like this

        Ok(result_id)
    }

    pub async fn update_movie(&self, movie: &Movie) -> Result<u64> {
        let movie_id = bson::to_bson(&movie.id)?.as_object_id().unwrap();

        let query_id = doc! {"_id":movie_id};
        let query_set = doc! {"$set":{"year":2019}};

        let update_result = self.movies.update_one(query_id, query_set, None).await?;

        let count = update_result.modified_count;

        Ok(count)
    }

    pub async fn movie_summary(&self, pipeline: Vec<Document>) -> Result<()> {
        let mut results = self.movies.aggregate(pipeline, None).await?;

        while let Some(result) = results.next().await {
            let doc: MovieSummary = bson::from_document(result?)?;
            println!("* {}", doc);
        }

        Ok(())
    }
}
