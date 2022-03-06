use axum::{
    routing::{get, post},
    Json, Router,
};
use tower_http::trace::TraceLayer;

pub fn app() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/json",
            post(|payload: Json<serde_json::Value>| async move {
                Json(serde_json::json!({"data":payload.0}))
            }),
        )
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
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
        Request::builder().uri("/").body(Body::empty()),
        StatusCode::OK
    );

    test_endpoint!(
        case_2,
        Request::builder()
            .method(Method::POST)
            .uri("/")
            .body(Body::empty()),
        StatusCode::METHOD_NOT_ALLOWED
    );

    test_endpoint!(
        case_3,
        Request::builder().uri("/404").body(Body::empty()),
        StatusCode::NOT_FOUND
    );

    test_endpoint!(
        case_4,
        Request::builder()
            .method(http::Method::POST)
            .uri("/json")
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(
                serde_json::to_vec(&json!([1, 2, 3, 4])).unwrap(),
            )),
        StatusCode::OK
    );
}
