mod error;
mod model;

use std::result;

use bson::oid::ObjectId;
use bson::Document;
use futures::StreamExt;
use model::Movie;

use chrono::{TimeZone, Utc};
use mongodb::bson::{doc, Bson};
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client, Collection,
};

use tokio;

use crate::model::MovieSummary;

type Result<T> = std::result::Result<T, error::Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let client = new_client().await?;

    let movies = client.database("sample_mflix").collection("movies");

    let parasite = Movie {
        id: None,
        title: "Parasite".to_string(),
        year: 2020,
        plot: "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.".to_string(),
        released: Utc.ymd(2020, 2, 7).and_hms(0, 0, 0),
    };

    let result_id = insert_one(&movies, &parasite).await?;

    let loaded_movie_struct = find_one(&movies, result_id).await?;

    update_one(&movies, &loaded_movie_struct).await?;

    aggregate(&movies).await?;

    Ok(())
}

pub async fn new_client() -> Result<Client> {
    let client_uri =
         "mongodb+srv://topheruk:VsNSZ28UcbGYJhw2@cluster0.pkfdw.mongodb.net/local?retryWrites=true&w=majority";

    let options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;
    let client = Client::with_options(options)?;

    Ok(client)
}

pub async fn aggregate(movies: &Collection<Document>) -> Result<()> {
    let stage_match_title = doc! {"$match": {"title": "A Star Is Born"}};

    let stage_sort_year_ascending = doc! {"$sort": { "year": 1 }};

    let pipeline = vec![stage_match_title, stage_sort_year_ascending];

    let mut results = movies.aggregate(pipeline, None).await?;

    while let Some(result) = results.next().await {
        let doc: MovieSummary = bson::from_document(result?)?;
        println!("* {}", doc);
    }

    Ok(())
}

pub async fn insert_one(movies: &Collection<Document>, movie: &Movie) -> Result<ObjectId> {
    let parasite = bson::to_bson(movie)?;

    let document = parasite.as_document().unwrap(); // safe to unwrap

    let insert_result = movies.insert_one(document.clone(), None).await?;
    let result_id = insert_result
        .inserted_id
        .as_object_id()
        .expect("Retrieved _id should have been of type ObjectId");
    println!("Parasite document ID: {:?}", result_id);

    Ok(result_id)
}

pub async fn find_one(movies: &Collection<Document>, result_id: ObjectId) -> Result<Movie> {
    let movie = movies
        .find_one(Some(doc! { "_id":  result_id.clone() }), None)
        .await?
        .expect("Document not found");

    let loaded_movie_struct = bson::from_bson::<Movie>(Bson::Document(movie))?;

    println!("Movie loaded from collection: {:?}", loaded_movie_struct);

    Ok(loaded_movie_struct)
}

pub async fn update_one(movies: &Collection<Document>, movie: &Movie) -> Result<()> {
    let serialized_movie_id = bson::to_bson(&movie.id)?
        .as_object_id()
        .expect("Retrieved _id should have been of type ObjectId"); // FIXME: no panic

    let query_id = doc! {"_id":serialized_movie_id};
    let query_set = doc! {"$set":{"year":2019}};

    let update_result = movies.update_one(query_id, query_set, None).await?;

    println!("Updated {} document", update_result.modified_count);

    Ok(())
}
