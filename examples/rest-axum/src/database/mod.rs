use axum::async_trait;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};
use uuid::Uuid;

pub mod error;

use error::Error;
type Result<T> = std::result::Result<T, Error>;

pub async fn client() -> Result<Client> {
    let client_uri =
        "mongodb+srv://topheruk:VsNSZ28UcbGYJhw2@cluster0.pkfdw.mongodb.net/local?retryWrites=true&w=majority";

    let options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;

    let cli = Client::with_options(options)?;

    Ok(cli)
}

#[async_trait]
pub trait Repo<T, C, U> {
    async fn find_all(&self) -> Result<Vec<T>>;
    async fn find(&self, id: Uuid) -> Result<T>;
    async fn create(&mut self, dto: C) -> Result<Uuid>;
    async fn delete(&mut self, id: Uuid) -> Result<()>;
    async fn update(&mut self, id: Uuid, dto: U) -> Result<T>;
}

pub mod movie;
pub mod todo;

#[cfg(test)]
mod test {
    use bson::doc;

    use crate::{
        database::client,
        model::{movie::Movie, todo::TodoCreate},
    };

    use super::{movie::MovieRepo, todo::TodoRepo, Repo, Result};

    #[tokio::test]
    async fn todo_repo_create() -> Result<()> {
        let mut repo = TodoRepo::default();

        let dto = TodoCreate {
            text: "My todo item".to_string(),
        };

        repo.create(dto).await?;

        Ok(())
    }

    #[tokio::test]
    async fn movie_repo_ping() -> Result<()> {
        // TODO: change this to Client::default()
        let client = client().await?;

        client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await?;
        println!("Connected successfully.");

        Ok(())
    }

    #[tokio::test]
    async fn movie_repo_find() -> Result<()> {
        let repo = MovieRepo::from(client().await?);

        if let Some(movie) = repo.find_one(doc! {"title":"One Week"}).await? {
            println!("{:?}", movie.plot);
        }

        Ok(())
    }

    #[tokio::test]
    async fn movie_repo_find_all() -> Result<()> {
        let repo = MovieRepo::from(client().await?);
        let movies = repo.find(None).await?;

        println!("{:?}", movies.len());
        Ok(())
    }

    #[tokio::test]
    async fn movie_repo_insert_one() -> Result<()> {
        let mut repo = MovieRepo::from(client().await?);

        let dto = Movie {
            id: None,
            title: "Parasite".to_string(),
            plot: Some("A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.".to_string()) 
        };

        let id = repo.insert_one(dto).await?;

        assert_ne!(id, None);
        Ok(())
    }
}
