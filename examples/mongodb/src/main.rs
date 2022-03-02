mod model;

use bson::oid::ObjectId;
use bson::Document;
use futures::StreamExt;
use model::Movie;

use chrono::{TimeZone, Utc};
use mongodb::bson::{self, doc, Bson};
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client, Collection,
};
use std::error::Error;
use tokio;

use crate::model::MovieSummary;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = new_client().await?;

    let movies = client.database("sample_mflix").collection("movies");

    let result_id = insert_one(&movies).await?;

    let loaded_movie_struct = find_one(&movies, result_id).await?;

    update_one(&movies, loaded_movie_struct).await?;

    aggregate(&movies).await?;

    Ok(())
}

// mongodb::error::Error

pub async fn new_client() -> Result<Client, Box<dyn Error>> {
    let client_uri =
         "mongodb+srv://topheruk:VsNSZ28UcbGYJhw2@cluster0.pkfdw.mongodb.net/local?retryWrites=true&w=majority";
    // env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");

    let options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;
    let client = Client::with_options(options)?;

    Ok(client)
}

// mongodb::error::Error
// bson::de::error::Error

pub async fn aggregate(movies: &Collection<Document>) -> Result<(), Box<dyn Error>> {
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

// bson::ser::error::Error
// mongodb::error::Error

pub async fn insert_one(movies: &Collection<Document>) -> Result<ObjectId, Box<dyn Error>> {
    let parasite = bson::to_bson(
        &Movie {
            id: None,
            title: "Parasite".to_string(),
            year: 2020,
            plot: "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.".to_string(),
            released: Utc.ymd(2020, 2, 7).and_hms(0, 0, 0),
        }
    )?;

    let document = parasite.as_document().unwrap(); // safe to unwrap

    let insert_result = movies.insert_one(document.clone(), None).await?;
    let result_id = insert_result
        .inserted_id
        .as_object_id()
        .expect("Retrieved _id should have been of type ObjectId");
    println!("Parasite document ID: {:?}", result_id);

    Ok(result_id)
}

// bson::de::error::Error
// mongodb::error::Error

pub async fn find_one(
    movies: &Collection<Document>,
    result_id: ObjectId,
) -> Result<Movie, Box<dyn Error>> {
    let movie = movies
        .find_one(Some(doc! { "_id":  result_id.clone() }), None)
        .await?
        .expect("Document not found");

    let loaded_movie_struct = bson::from_bson::<Movie>(Bson::Document(movie))?;

    println!("Movie loaded from collection: {:?}", loaded_movie_struct);

    Ok(loaded_movie_struct)
}

// bson::ser::error::Error
// mongodb::error::Error

pub async fn update_one(
    movies: &Collection<Document>,
    loaded_movie_struct: Movie,
) -> Result<(), Box<dyn Error>> {
    let serialized_movie_id = bson::to_bson(&loaded_movie_struct.id)?
        .as_object_id()
        .expect("Retrieved _id should have been of type ObjectId");

    let update_result = movies
        .update_one(
            doc! {
               "_id": serialized_movie_id
            },
            doc! {
               "$set": { "year": 2019 }
            },
            None,
        )
        .await?;

    println!("Updated {} document", update_result.modified_count);

    Ok(())
}
