mod app;

use crate::app::handler::*;

use actix_web::{self, App, HttpServer};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    HttpServer::new(|| App::new().service(json))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use actix_web::{body, test, App};

    use super::*;

    #[actix_web::test]
    async fn test_json() {
        let app = test::init_service(App::new().service(json)).await;
        let req = test::TestRequest::get().uri("/a/John").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let bytes = body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(bytes, r##"{"name":"John"}"##);
    }

    #[actix_web::test]
    async fn test_ping() {
        let app = test::init_service(App::new().service(ping)).await;
        let req = test::TestRequest::get().uri("/ping").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let bytes = body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(bytes, r##"ping"##);
    }
}
