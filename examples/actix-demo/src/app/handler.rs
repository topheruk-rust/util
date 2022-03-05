use actix_web::{
    get,
    web::{Json, Path},
    Result,
};

use serde::Serialize;

#[derive(Serialize)]
pub struct MyObj {
    pub name: String,
}

#[get("/ping")]
pub async fn ping() -> String {
    "ping".to_string()
}

#[get("/a/{name}")]
pub async fn json(name: Path<String>) -> Result<Json<MyObj>> {
    let obj = MyObj {
        name: name.to_string(),
    };

    Ok(Json(obj))
}
