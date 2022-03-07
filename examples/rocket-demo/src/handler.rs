use rocket::get;

#[get("/world")]
pub fn world() -> String {
    "Hello, World!".to_string()
}
