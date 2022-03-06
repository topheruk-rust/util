mod model {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    pub type Database = Arc<RwLock<HashMap<Uuid, Todo>>>;

    #[derive(Debug, Serialize, Clone)]
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
    }

    impl From<TodoDto> for Todo {
        fn from(TodoDto { text }: TodoDto) -> Self {
            Self::new(text)
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct TodoDto {
        pub text: String,
    }

    #[derive(Debug, Deserialize, Default)]
    pub struct Pagination {
        pub page: usize,
        pub per_page: usize,
    }
}

mod handler {
    use axum::{
        extract::{Extension, Path, Query},
        http::StatusCode,
        response::IntoResponse,
        Json,
    };
    use uuid::Uuid;

    use crate::model::{Database, Pagination, Todo, TodoDto};

    pub async fn todos_index(
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

        (StatusCode::OK, Json(todos))
    }

    pub async fn todos_create(
        Json(input): Json<TodoDto>,
        Extension(db): Extension<Database>,
    ) -> impl IntoResponse {
        let todo: Todo = input.into();

        db.write().unwrap().insert(todo.id, todo.clone());

        (StatusCode::CREATED, Json(todo))
    }

    pub async fn todos_delete(
        Path(id): Path<Uuid>,
        Extension(db): Extension<Database>,
    ) -> impl IntoResponse {
        if let Some(_) = db.write().unwrap().remove(&id) {
            StatusCode::NO_CONTENT
        } else {
            StatusCode::NOT_FOUND
        }
    }
}

use axum::{
    extract::Extension,
    routing::{delete, get},
    Router,
};
use handler::{todos_create, todos_delete, todos_index};
use model::Database;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

pub fn app() -> Router {
    let db = Database::default();

    Router::new()
        .route("/todos/", get(todos_index).post(todos_create))
        .route("/todos/:id", delete(todos_delete))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(Extension(db))
                .into_inner(),
        )
}

#[cfg(test)]
mod tests {
    use crate::model::TodoDto;

    use super::*;
    use axum::{
        body::Body,
        http::{self, Method, Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt; // for `app.oneshot()`

    macro_rules! test_endpoint {
        ($n:ident, $req:expr, $sc:expr) => {
            #[tokio::test]
            async fn $n() {
                let app = app();
                let resp = app.oneshot($req.unwrap()).await.unwrap();

                assert_eq!(resp.status(), $sc);
            }
        };
    }

    test_endpoint!(
        case_1,
        Request::builder()
            .method(Method::POST)
            .uri("/todos/")
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(
                serde_json::to_vec(&json!({"text":"Hello, World"})).unwrap(),
            )),
        StatusCode::CREATED
    );

    // test_endpoint!(
    //     case_2,
    //     Request::builder()
    //         .method(Method::POST)
    //         .uri("/")
    //         .body(Body::empty()),
    //     StatusCode::METHOD_NOT_ALLOWED
    // );

    // test_endpoint!(
    //     case_3,
    //     Request::builder().uri("/404").body(Body::empty()),
    //     StatusCode::NOT_FOUND
    // );

    // test_endpoint!(
    //     case_4,
    //     Request::builder()
    //         .method(http::Method::POST)
    //         .uri("/json")
    //         .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    //         .body(Body::from(
    //             serde_json::to_vec(&json!([1, 2, 3, 4])).unwrap(),
    //         )),
    //     StatusCode::OK
    // );
}
