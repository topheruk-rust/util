use actix_web::{
    get,
    web::{Json, Path},
    Responder, Result,
};

use serde::Serialize;

#[derive(Serialize)]
pub struct MyObj {
    pub name: String,
}

#[get("/a/{name}")]
pub async fn json(name: Path<String>) -> Result<Json<MyObj>> {
    let obj = MyObj {
        name: name.to_string(),
    };

    Ok(Json(obj))
}
