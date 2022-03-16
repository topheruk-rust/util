use std::net::SocketAddr;

use axum::{extract::Extension, routing::get, Router, Server};
use database::{client, movie::MovieRepo, todo::TodoRepo};
use handler::{movies_index, todos_create, todos_delete, todos_index, todos_member, todos_update};

// some possible visibility restrictions are:
// `pub(crate)`: visible only on the current crate
// `pub(super)`: visible only in the current module's parent
// `pub(in path::to::module)`: visible only on the specified path

pub mod error {
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
                Error::Database(DbError::NotFound) => {
                    (StatusCode::NOT_FOUND, "not found".to_string())
                }
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
                Error::Database(DbError::Deserialize(e)) => {
                    (StatusCode::BAD_REQUEST, e.to_string())
                }
                Error::Database(DbError::Serialize(e)) => (StatusCode::BAD_REQUEST, e.to_string()),
            };

            let body = Json(json!({ "error": message }));

            (code, body).into_response()
        }
    }
}

pub mod model {
    use self::error::Error;

    pub(super) mod error {
        #[derive(Debug)]
        pub enum Error {
            Serde(serde_json::Error),
            EmptyText,
            BsonSerialize(bson::ser::Error),
            BsonDeserialize(bson::de::Error),
        }

        impl From<serde_json::Error> for Error {
            fn from(e: serde_json::Error) -> Self {
                Self::Serde(e)
            }
        }

        impl From<bson::ser::Error> for Error {
            fn from(e: bson::ser::Error) -> Self {
                Self::BsonSerialize(e)
            }
        }

        impl From<bson::de::Error> for Error {
            fn from(e: bson::de::Error) -> Self {
                Self::BsonDeserialize(e)
            }
        }
    }

    type Result<T> = std::result::Result<T, Error>;

    pub(super) mod todo {
        use serde::{Deserialize, Serialize};
        use uuid::Uuid;

        use crate::model::error::Error;
        use crate::model::Result;

        #[derive(Debug, Serialize, Clone)]
        pub struct Todo {
            pub id: Uuid,
            pub text: String,
            pub completed: bool,
        }

        impl Todo {
            // TODO:  what if empty string
            // non lic should prevent this being an issue
            fn new(text: String) -> Self {
                Self {
                    id: Uuid::new_v4(),
                    text,
                    completed: false,
                }
            }
        }

        impl TryFrom<TodoCreate> for Todo {
            type Error = Error;

            fn try_from(value: TodoCreate) -> Result<Self> {
                let TodoCreate { text } = match value {
                    _ if value.text.is_empty() => Err(Error::EmptyText),
                    _ => Ok(value),
                }?;

                Ok(Todo::new(text))
            }
        }

        #[derive(Debug, Deserialize)]
        pub struct TodoUpdate {
            pub text: Option<String>,
            pub completed: Option<bool>,
        }

        #[derive(Debug, Deserialize)]
        pub struct TodoCreate {
            pub text: String,
        }
    }

    pub(super) mod movie {
        use std::fmt;

        // use chrono::Utc;
        use mongodb::bson::oid::ObjectId;
        use serde::{Deserialize, Serialize};

        // You use `serde` to create structs which can serialize & deserialize between BSON:
        #[derive(Serialize, Deserialize, Debug, Clone)]
        pub struct Movie {
            #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
            pub id: Option<ObjectId>,
            pub title: String,
            // pub year: i64, // FIXME: invalid type
            pub plot: Option<String>,
            // #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
            // pub released: chrono::DateTime<Utc>, // FIXME: does not work
        }

        #[derive(Deserialize)]
        pub struct MovieSummary {
            pub title: String,
            pub cast: Vec<String>,
            pub year: i32,
        }

        impl fmt::Display for MovieSummary {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{}, {}, {}",
                    self.title,
                    self.cast.get(0).unwrap_or(&"- no cast -".to_owned()),
                    self.year
                )
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::model::{
            todo::{Todo, TodoCreate},
            Result,
        };

        macro_rules! test_todo_create {
            ($name:ident, $str:expr) => {
                #[test]
                fn $name() {
                    let t = TodoCreate {
                        text: $str.to_string(),
                    };

                    match t.try_into() as Result<Todo> {
                        Ok(t) => assert_eq!(t.completed, false),
                        Err(_) => assert!(true), // Err(e) => assert_eq!(e, error::Error::EmptyText),
                    };
                }
            };
        }

        test_todo_create!(todo_create_todo_ok, "John");
        test_todo_create!(test_create_todo_err, "");

        #[test]
        fn test_todo_json() -> Result<()> {
            let t = serde_json::from_str::<TodoCreate>(r#"{"text":"Hello, World"}"#)?;
            let t: Todo = t.try_into()?;
            assert_eq!(t.completed, false);
            Ok(())
        }
    }
}

pub mod database {
    use axum::async_trait;
    use mongodb::{
        options::{ClientOptions, ResolverConfig},
        Client,
    };
    use uuid::Uuid;

    use self::error::Error;

    pub(super) mod error {
        use crate::model::error::Error as ModelError;

        #[derive(Debug)]
        pub enum Error {
            NotFound,
            Model(ModelError),
            Mongo(mongodb::error::Error),
            Serialize(bson::ser::Error),
            Deserialize(bson::de::Error),
        }

        impl From<ModelError> for Error {
            fn from(e: ModelError) -> Self {
                Self::Model(e)
            }
        }

        impl From<mongodb::error::Error> for Error {
            fn from(err: mongodb::error::Error) -> Self {
                Self::Mongo(err)
            }
        }

        impl From<bson::ser::Error> for Error {
            fn from(e: bson::ser::Error) -> Self {
                Self::Serialize(e)
            }
        }

        impl From<bson::de::Error> for Error {
            fn from(e: bson::de::Error) -> Self {
                Self::Deserialize(e)
            }
        }
    }

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
    pub(super) trait Repo<T, C, U> {
        async fn find_all(&self) -> Result<Vec<T>>;
        async fn find(&self, id: Uuid) -> Result<T>;
        async fn create(&mut self, dto: C) -> Result<Uuid>;
        async fn delete(&mut self, id: Uuid) -> Result<()>;
        async fn update(&mut self, id: Uuid, dto: U) -> Result<T>;
    }

    pub mod movie {
        use bson::{oid::ObjectId, Document};
        use futures::TryStreamExt;
        use mongodb::Collection;

        use crate::model::movie::Movie;

        use super::Result;

        #[derive(Debug, Clone)]
        pub struct MovieRepo {
            db: Collection<Movie>,
            // options
        }

        impl From<mongodb::Client> for MovieRepo {
            fn from(cli: mongodb::Client) -> Self {
                Self {
                    db: cli.database("sample_mflix").collection("movies"),
                }
            }
        }

        impl MovieRepo {
            pub async fn find(
                &self,
                pipeline: impl IntoIterator<Item = Document>,
            ) -> Result<Vec<Movie>> {
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
    }

    pub mod todo {
        use std::{
            collections::HashMap,
            sync::{Arc, RwLock},
        };

        use axum::async_trait;
        use uuid::Uuid;

        use crate::{
            database::{Error, Repo, Result},
            model::todo::{Todo, TodoCreate, TodoUpdate},
        };

        #[derive(Debug, Clone)]
        pub struct TodoRepo {
            db: Arc<RwLock<HashMap<Uuid, Todo>>>,
        }

        impl TodoRepo {
            pub fn new() -> Self {
                Self { db: Arc::default() }
            }
        }

        #[async_trait]
        impl Repo<Todo, TodoCreate, TodoUpdate> for TodoRepo {
            async fn find(&self, id: Uuid) -> Result<Todo> {
                let todos = self.db.read().unwrap(); // FIXME: say no to unwrap!!
                let todo = todos.get(&id);

                match todo {
                    Some(todo) => Ok(todo.clone()),
                    None => Err(Error::NotFound),
                }
            }

            async fn find_all(&self) -> Result<Vec<Todo>> {
                let todos = self.db.read().unwrap(); // FIXME: say no to unwrap!!
                let todos = todos.values().cloned().collect::<Vec<_>>();

                Ok(todos)
            }

            async fn create(&mut self, dto: TodoCreate) -> Result<Uuid> {
                let todo: Todo = dto.try_into()?;
                let id = todo.id;

                let mut todos = self.db.write().unwrap(); // FIXME: say no to unwrap!!
                todos.insert(id, todo);

                Ok(id)
            }

            async fn delete(&mut self, id: Uuid) -> Result<()> {
                let mut todos = self.db.write().unwrap();

                match todos.remove(&id).is_some() {
                    true => Ok(()),
                    false => Err(Error::NotFound),
                }
            }

            async fn update(&mut self, id: Uuid, dto: TodoUpdate) -> Result<Todo> {
                let mut todo = self.find(id).await?;

                if let Some(text) = dto.text {
                    todo.text = text;
                }

                if let Some(completed) = dto.completed {
                    todo.completed = completed;
                }

                let mut todos = self.db.write().unwrap(); // FIXME: say no to unwrap!!

                todos.insert(todo.id, todo.clone());

                Ok(todo)
            }
        }
    }

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
            let mut repo = TodoRepo::new();

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
}

pub mod handler {
    use axum::{
        extract::{Extension, Path, Query},
        http::StatusCode,
        Json,
    };
    use bson::doc;
    use serde::Deserialize;
    use serde_json::{json, Value};
    use uuid::Uuid;

    use crate::{
        database::{movie::MovieRepo, todo::TodoRepo, Repo},
        model::{
            movie::Movie,
            todo::{Todo, TodoCreate, TodoUpdate},
        },
    };

    use crate::error::Result;

    pub async fn todos_index(Extension(repo): Extension<TodoRepo>) -> Result<Json<Vec<Todo>>> {
        let todos = repo.find_all().await?;

        Ok(Json(todos))
    }

    pub async fn todos_member(
        Path(id): Path<Uuid>,
        Extension(repo): Extension<TodoRepo>,
    ) -> Result<Json<Todo>> {
        let todo = repo.find(id).await?;

        Ok(Json(todo))
    }

    pub async fn todos_create(
        Json(dto): Json<TodoCreate>,
        Extension(mut repo): Extension<TodoRepo>,
    ) -> Result<(StatusCode, Json<Value>)> {
        let uid = repo.create(dto).await?;

        Ok((StatusCode::CREATED, Json(json!({ "id": uid }))))
    }

    pub async fn todos_update(
        Path(id): Path<Uuid>,
        Json(dto): Json<TodoUpdate>,
        Extension(mut repo): Extension<TodoRepo>,
    ) -> Result<(StatusCode, Json<Todo>)> {
        let todo = repo.update(id, dto).await?;

        Ok((StatusCode::CREATED, Json(todo)))
    }

    pub async fn todos_delete(
        Path(id): Path<Uuid>,
        Extension(mut repo): Extension<TodoRepo>,
    ) -> Result<StatusCode> {
        repo.delete(id).await?;

        Ok(StatusCode::NO_CONTENT)
    }

    // -- movies

    #[derive(Debug, Deserialize, Default)]
    pub struct Pagination {
        pub offset: Option<u32>,
        pub limit: Option<u32>, // this will be too large
    }

    pub async fn movies_index(
        pagination: Option<Query<Pagination>>,
        Extension(repo): Extension<MovieRepo>,
    ) -> Result<Json<Vec<Movie>>> {
        let Query(Pagination { offset, limit }) = pagination.unwrap_or_default();

        let skip = offset.unwrap_or(0);
        let stage_skip = doc! { "$skip": skip };

        // if number is greater than `MAX` then default
        let limit = limit.unwrap_or(20);
        let stage_limit = doc! { "$limit": limit };

        let todos = repo.find(vec![stage_skip, stage_limit]).await?;
        Ok(Json(todos))
    }
}

// #[tokio::as]
async fn app() -> Router {
    let todos = TodoRepo::new();
    let movies = MovieRepo::from(client().await.unwrap());

    Router::new()
        .route("/movies/", get(movies_index))
        .route("/todos/", get(todos_index).post(todos_create))
        .route(
            "/todos/:id",
            get(todos_member).patch(todos_update).delete(todos_delete),
        )
        .layer(Extension(movies))
        .layer(Extension(todos))
}

#[tokio::main]
async fn main() {
    let app = app();

    let addr = &SocketAddr::from(([127, 0, 0, 1], 3000));

    Server::bind(addr)
        .serve(app.await.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use axum::http::{header, Method, Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt; // for `app.oneshot()`

    macro_rules! test_post {
        ($test_name:ident, $uri:expr, $json:expr, $status_code:expr) => {
            #[tokio::test]
            async fn $test_name() {
                let app = app();
                let req = Request::builder()
                    .method(Method::POST)
                    .uri($uri)
                    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(serde_json::to_vec(&$json).unwrap().into())
                    .unwrap();

                let res = app.await.oneshot(req).await.unwrap();
                assert_eq!(res.status(), $status_code);
            }
        };
    }

    test_post!(
        case_created,
        "/todos/",
        json!({"text":""}),
        StatusCode::BAD_REQUEST
    );
}
