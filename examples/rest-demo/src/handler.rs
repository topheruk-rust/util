use actix_web::get;

// #[get("/hello")]
pub async fn index() -> String {
    "Hello, World!".to_string()
}
