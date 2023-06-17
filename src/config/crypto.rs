use crate::models::claim::Claims;
use argon2::{self, Config};
use chrono::{DateTime, Utc};
use color_eyre::Result;
use eyre::eyre;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use std::sync::Arc;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct CryptoService {
    pub key: Arc<String>,
}

impl CryptoService {
    #[instrument]
    pub async fn hash_password(&self, password: String) -> Result<String> {
        let config = Config::default();
        argon2::hash_encoded(password.as_bytes(), self.key.as_bytes(), &config)
            .map_err(|err| eyre!("Hashing error: {:?}", err))
    }

    #[instrument]
    pub fn generate_jwt(&self, username: String) -> Result<(String, DateTime<Utc>)> {
        let iat = Utc::now();
        let exp = iat + chrono::Duration::hours(1);

        let claims = Claims::new(username, iat, exp);

        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.key.as_bytes()),
        )
        .map(|res| (res, exp))
        .map_err(|err| eyre!("JWT Encoding error: {:?}", err))
    }

    #[instrument]
    pub fn decode_jwt(&self, token: String) -> Result<Claims> {
        jsonwebtoken::decode::<Claims>(
            &token,
            &DecodingKey::from_secret(self.key.as_bytes()),
            &Validation::default(),
        )
        .map(|token_data| token_data.claims)
        .map_err(|err| eyre!("JWT Decoding error: {:?}", err))
    }
}
