use axum::{extract::Extension, routing::get, Router};
use database::{client, movie::MovieRepo, todo::TodoRepo};
use handler::{movies_index, todos_create, todos_delete, todos_index, todos_member, todos_update};

// some possible visibility restrictions are:
// `pub(crate)`: visible only on the current crate
// `pub(super)`: visible only in the current module's parent
// `pub(in path::to::module)`: visible only on the specified path

mod database;
mod error;
mod handler;
mod model;

// #[tokio::as]
pub async fn app() -> Router {
    let todos = TodoRepo::default();
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

// #[tokio::main]
// async fn main() {
//     let app = app();

//     let addr = &SocketAddr::from(([127, 0, 0, 1], 3000));

//     Server::bind(addr)
//         .serve(app.await.into_make_service())
//         .await
//         .unwrap();
// }

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
