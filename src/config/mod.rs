pub mod crypto;

use crate::config::crypto::CryptoService;
use crate::db::user::UserRepository;
use crate::graphql::schema::{AppSchema, Mutation, Query};
use async_graphql::{EmptySubscription, Schema};
use color_eyre::Result;
use dotenv::dotenv;
use eyre::WrapErr;
use serde::Deserialize;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, instrument};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: i32,
    pub database_url: String,
    pub secret_key: String,
    pub github_client_id: String,
    pub github_client_secret: String,
}

impl Config {
    #[instrument]
    pub fn from_env(enable_tracing: bool) -> Result<Config> {
        dotenv().ok();

        if enable_tracing {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        }

        info!("Loading configuration");

        let mut c = config::Config::new();

        c.merge(config::Environment::default())?;

        c.try_into()
            .context("Loading configuration from environment")
    }

    pub async fn new_db_pool(&self) -> Result<MySqlPool> {
        info!("Creating database connection pool.");

        MySqlPoolOptions::new()
            .connect_timeout(Duration::from_secs(30))
            .connect(&self.database_url)
            .await
            .context("creating database connection pool")
    }

    pub fn new_crypto_service(&self) -> CryptoService {
        CryptoService {
            key: Arc::new(self.secret_key.clone()),
        }
    }

    pub fn new_schema(
        &self,
        user_repo: UserRepository,
        crypto_service: CryptoService,
        http_client: reqwest::Client,
    ) -> AppSchema {
        Schema::build(Query, Mutation, EmptySubscription)
            .data(user_repo)
            .data(crypto_service)
            .data(http_client)
            .finish()
    }
}
