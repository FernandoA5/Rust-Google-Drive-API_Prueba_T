mod auth;
mod handlers;
mod state;

use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;
use actix_cors::Cors;

#[cfg(test)]
mod tests;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let hub = auth::get_drive_hub().await;

    let app_state = actix_web::web::Data::new(state::AppState {
        hub: tokio::sync::Mutex::new(hub),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin() // Aquí puedes especificar dominios permitidos
                    .allow_any_method()
                    .allow_any_header()
            )
            .service(handlers::list_files)
            .service(handlers::upload_file)
            .service(handlers::download_file)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}