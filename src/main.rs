mod auth;
mod groups;
mod http_responses;
mod models;
mod routes;
mod schema;

use actix_web::{App, HttpServer, web};
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use log::info;
use std::env;

use models::init_admin_user;
use routes::{auth_routes, file_routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init(); // inicializa o logger

    info!("Iniciando servidor...");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    // Cria usuário admin padrão
    init_admin_user(&mut pool.get().unwrap());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(auth_routes)
            .configure(file_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
