mod app;

use actix_web::{
    self,
    web::{self, Data},
    App, HttpServer,
};
use app::{
    database::AppState,
    handler::{echo, hello, manual_hello},
    Result,
};

#[actix_web::main]
async fn main() -> Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(AppState {
                app_name: "Actix-web".to_string(),
            }))
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
