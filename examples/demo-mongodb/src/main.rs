mod database;
mod error;
mod model;
mod result;

use crate::{database::Repo, model::Movie, result::Result};

use chrono::{TimeZone, Utc};
use mongodb::bson::{doc, Bson};
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};

#[tokio::main]
async fn main() -> Result<()> {
    let repo = database::Repo::from(client().await?);

    let parasite = Movie {
        id: None,
        title: "Parasite".to_string(),
        year: 2020,
        plot: "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.".to_string(),
        released: Utc.ymd(2020, 2, 7).and_hms(0, 0, 0),
    };

    let result_id = repo.insert_movie(&parasite).await?;

    let loaded_movie_struct = repo.find_movie(result_id).await?;

    let modified = repo.update_movie(&loaded_movie_struct).await?;

    let stage_match_title = doc! {"$match": {"title": "A Star Is Born"}};

    let stage_sort_year_ascending = doc! {"$sort": { "year": 1 }};

    let pipeline = vec![stage_match_title, stage_sort_year_ascending];
    repo.movie_summary(pipeline).await?;

    Ok(())
}

pub async fn client() -> Result<Client> {
    let client_uri =
         "mongodb+srv://topheruk:VsNSZ28UcbGYJhw2@cluster0.pkfdw.mongodb.net/local?retryWrites=true&w=majority";

    let options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;
    let client = Client::with_options(options)?;

    Ok(client)
}
