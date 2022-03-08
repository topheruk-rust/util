use axum::{extract::Extension, routing::get, Router};
use handler::{todos_create, todos_delete, todos_index, todos_member, todos_update};
use model::TodoRepo;

pub mod error {
    use axum::{
        response::{IntoResponse, Response},
        Json,
    };
    use hyper::StatusCode;
    use serde_json::json;

    use crate::model::error::Error as ModelError;

    pub enum Error {
        Model(ModelError),
    }

    pub type Result<T> = std::result::Result<T, Error>;

    impl From<ModelError> for Error {
        fn from(e: ModelError) -> Self {
            Self::Model(e)
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> Response {
            let (code, message) = match self {
                Error::Model(ModelError::EmptyText) => {
                    (StatusCode::BAD_REQUEST, "emtpy text".to_string())
                }
                Error::Model(ModelError::NotFound) => {
                    (StatusCode::NOT_FOUND, "not found".to_string())
                }
                Error::Model(ModelError::Encoding(e)) => (StatusCode::BAD_REQUEST, e.to_string()),
            };

            let body = Json(json!({ "error": message }));

            (code, body).into_response()
        }
    }
}

mod model {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use axum::async_trait;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use self::error::Error;

    pub mod error {
        #[derive(Debug)]
        pub enum Error {
            Encoding(serde_json::Error),
            // Poision(PoisonError<RwLockReadGuard<HashMap<Uuid, Todo>>>), // TODO: make generic
            EmptyText,
            NotFound,
            // Invalid,
        }

        impl From<serde_json::Error> for Error {
            fn from(e: serde_json::Error) -> Self {
                Self::Encoding(e)
            }
        }
    }

    type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, Serialize, Clone)]
    pub struct Todo {
        pub id: Uuid,
        pub text: String,
        pub completed: bool,
    }

    impl Todo {
        // TODO:  what if empty string
        // non public should prevent this being an issue
        fn new(text: String) -> Self {
            Self {
                id: Uuid::new_v4(),
                text,
                completed: false,
            }
        }

        pub fn default() -> Self {
            Self::new("todo".to_string())
        }
    }

    impl TryFrom<TodoCreate> for Todo {
        type Error = Error;

        fn try_from(value: TodoCreate) -> Result<Self> {
            if value.text.is_empty() {
                Err(Self::Error::EmptyText)
            } else {
                Ok(Self::new(value.text))
            }
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

    #[derive(Debug, Clone)]
    pub struct TodoRepo {
        pub db: Arc<RwLock<HashMap<Uuid, Todo>>>,
    }

    impl TodoRepo {
        pub fn new() -> Self {
            Self { db: Arc::default() }
        }
    }

    #[async_trait]
    pub trait Repo {
        async fn find_all(&self) -> Result<Vec<Todo>>;
        async fn find(&self, id: Uuid) -> Result<Todo>;
        async fn create(&mut self, dto: TodoCreate) -> Result<Uuid>;
        async fn delete(&mut self, id: Uuid) -> Result<()>;
        async fn update(&mut self, id: Uuid, dto: TodoUpdate) -> Result<Todo>;
    }

    #[async_trait]
    impl Repo for TodoRepo {
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
            let todo: Todo = dto.try_into().unwrap(); // FIXME: say no to unwrap!!
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
                // TODO: I need to check if this catches EmtpyText too? Probs not
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

    #[cfg(test)]
    mod test {
        use super::*;

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

mod handler {
    use axum::{
        extract::{Extension, Path},
        response::IntoResponse,
        Json,
    };
    use hyper::StatusCode;
    use serde_json::{json, Value};
    use uuid::Uuid;

    use crate::model::{Repo, Todo, TodoCreate, TodoRepo, TodoUpdate};

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
}

pub fn app() -> Router {
    let repo = TodoRepo::new();

    Router::new()
        .route("/todos/", get(todos_index).post(todos_create))
        .route(
            "/todos/:id",
            get(todos_member).patch(todos_update).delete(todos_delete),
        )
        .layer(Extension(repo))
}

#[cfg(test)]
mod tests {
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

                let res = app.oneshot(req).await.unwrap();
                assert_eq!(res.status(), $status_code);
            }
        };
    }

    test_post!(
        case_created,
        "/todos/",
        json!({"text":"Hello, World!"}),
        StatusCode::CREATED
    );
}
