use actix_web::{get, post, web::Data, HttpResponse, Responder};

use crate::app::database::AppState;

#[get("/")]
pub async fn hello(data: Data<AppState>) -> String {
    let app_name = &data.app_name;
    format!("Hello, {}!", app_name)
}

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

pub async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
