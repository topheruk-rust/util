mod model;

use model::Movie;

use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};
use std::error::Error;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load the MongoDB connection string from an environment variable:
    let client_uri =
         "mongodb+srv://topheruk:VsNSZ28UcbGYJhw2@cluster0.pkfdw.mongodb.net/local?retryWrites=true&w=majority";
    // env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");

    // A Client is needed to connect to MongoDB:
    // An extra line of code to work around a DNS issue on Windows:
    let options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;
    let client = Client::with_options(options)?;

    // -- print name sof all databases
    println!("Databases:");
    for name in client.list_database_names(None, None).await? {
        println!("- {}", name);
    }

    // -- create a doc
    use chrono::{TimeZone, Utc};
    use mongodb::bson::{self, doc, Bson};

    let parasite = Movie {
        id: None,
        title: "Parasite".to_string(),
        year: 2020,
        plot: "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.".to_string(),
        released: Utc.ymd(2020, 2, 7).and_hms(0, 0, 0),
    };

    // -- add to collection
    // convert to bson instance
    let serialized_movie = bson::to_bson(&parasite)?;
    let document = serialized_movie.as_document().unwrap(); // safe to unwrap

    let movies = client.database("sample_mflix").collection("movies");
    let insert_result = movies.insert_one(document.clone(), None).await?;
    let parasite_id = insert_result
        .inserted_id
        .as_object_id()
        .expect("Retrieved _id should have been of type ObjectId");
    println!("Captain Marvel document ID: {:?}", parasite_id);

    // -- retrieve document
    let movie = movies
        .find_one(Some(doc! { "_id":  parasite_id.clone() }), None)
        .await?
        .expect("Document not found");
    let loaded_movie_struct = bson::from_bson::<Movie>(Bson::Document(movie))?;
    println!("Movie loaded from collection: {:?}", loaded_movie_struct);

    // -- updating a document
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
