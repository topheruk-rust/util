mod error {
    use crate::model;

    pub enum AppError {
        RepoError(model::RepoError),
    }

    impl From<model::RepoError> for AppError {
        fn from(e: model::RepoError) -> Self {
            Self::RepoError(e)
        }
    }
}

mod model {
    mod error {
        pub enum RepoError {
            NotFound,
        }

        struct Repo {
            db: Database,
        }

        impl Repo {
            pub fn get(&self, id: Uuid) -> Result<Todo, RepoError> {
                match self.db.read().unwrap().get(&id) {
                    Some(todo) => Ok(todo.clone()),
                    _ => Err(RepoError::NotFound),
                }
            }
        }
    }

    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use axum::async_trait;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    pub type Database = Arc<RwLock<HashMap<Uuid, Todo>>>;

    #[derive(Serialize, Clone)]
    pub struct Todo {
        pub id: Uuid,
        pub text: String,
        pub completed: bool,
    }

    impl Todo {
        pub fn new(text: String) -> Self {
            Self {
                id: Uuid::new_v4(),
                text,
                completed: false,
            }
        }

        pub fn default() -> Self {
            Self {
                id: Uuid::default(),
                text: "".to_string(),
                completed: false,
            }
        }
    }

    impl From<TodoDto> for Todo {
        fn from(TodoDto { text }: TodoDto) -> Self {
            Self::new(text)
        }
    }

    #[derive(Deserialize)]
    pub struct TodoDto {
        pub text: String,
    }

    #[derive(Deserialize, Default)]
    pub struct Pagination {
        pub page: usize,
        pub per_page: usize,
    }
}

mod handler {
    use axum::{
        extract::{Extension, Query},
        http::StatusCode,
        response::IntoResponse,
        Json,
    };
    use axum_extra::routing::TypedPath;
    use serde::Deserialize;
    use uuid::Uuid;

    use crate::model::{Database, Pagination, Todo, TodoDto};

    #[derive(Deserialize, TypedPath)]
    #[typed_path("/todos/")]
    pub struct TodosCollection;

    pub async fn todos_index(
        _: TodosCollection,
        Query(Pagination { page, per_page }): Query<Pagination>,
        Extension(db): Extension<Database>,
    ) -> impl IntoResponse {
        let todos = db.read().unwrap();

        let todos = todos
            .values()
            .skip(page)
            .take(per_page)
            .cloned()
            .collect::<Vec<_>>();

        Json(todos)
    }

    pub async fn todos_create(
        _: TodosCollection,
        Json(input): Json<TodoDto>,
        Extension(db): Extension<Database>,
    ) -> (StatusCode, Json<Todo>) {
        let todo: Todo = input.into();

        db.write().unwrap().insert(todo.id, todo.clone());

        (StatusCode::CREATED, Json(todo))
    }

    #[derive(Deserialize, TypedPath)]
    #[typed_path("/todos/:id")]
    pub struct TodosMember {
        id: Uuid,
    }

    pub async fn todos_delete(
        TodosMember { id }: TodosMember,
        Extension(db): Extension<Database>,
    ) -> impl IntoResponse {
        if let Some(_) = db.write().unwrap().remove(&id) {
            StatusCode::NO_CONTENT
        } else {
            StatusCode::NOT_FOUND
        }
    }
}

use axum::{extract::Extension, Router};
use axum_extra::routing::RouterExt;
use handler::{todos_create, todos_delete, todos_index};
use model::Database;

pub fn app() -> Router {
    let todos_repo = Database::default();

    Router::new()
        .typed_get(todos_index)
        .typed_post(todos_create)
        .typed_delete(todos_delete)
        .layer(Extension(todos_repo))
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
