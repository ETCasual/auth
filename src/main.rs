#[macro_use]
extern crate validator_derive;
#[macro_use]
extern crate lazy_static;

mod config;
mod db;
mod error;
mod graphql;
mod handlers;
mod models;

use crate::config::Config;
use crate::db::user::UserRepository;
use actix_web::{middleware::Logger, web, App, HttpServer};
use color_eyre::Result;
use handlers::app_config;
use std::sync::Arc;
use tracing::info;

// This is an immutable application state
struct AppState {
    app_name: String,
    app_config: Config,
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let config = Config::from_env(true).expect("Server configuration");

    let pool = config.new_db_pool().await.expect("Database configuration");

    let crypto_service = config.new_crypto_service();

    let user_repository = UserRepository::new(Arc::new(pool.clone()));

    let http_client = reqwest::Client::new();

    let schema = config.new_schema(
        user_repository.clone(),
        crypto_service.clone(),
        http_client.clone(),
    );

    let host = config.host.clone();
    let port = config.port.clone();

    info!("Starting server at http://{}:{}", config.host, config.port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(AppState {
                app_name: String::from("Auth"),
                app_config: config.clone(),
            }))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(crypto_service.clone()))
            .app_data(web::Data::new(user_repository.clone()))
            .app_data(web::Data::new(http_client.clone()))
            .app_data(web::Data::new(schema.clone()))
            .configure(app_config)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;

    Ok(())
}

