use async_graphql::Context;

use crate::{config::crypto::CryptoService, db::user::UserRepository};

pub fn get_user_repo_from_ctx(ctx: &Context<'_>) -> UserRepository {
    ctx.data::<UserRepository>()
        .expect("Can't get UserRepository from GraphQL Context")
        .clone()
}

pub fn get_crypto_service_from_ctx(ctx: &Context<'_>) -> CryptoService {
    ctx.data::<CryptoService>()
        .expect("Can't get CryptoService from GraphQL Context")
        .clone()
}

pub fn get_http_client_from_ctx(ctx: &Context<'_>) -> reqwest::Client {
    ctx.data::<reqwest::Client>()
        .expect("Can't get HttpClient from GraphQL Context")
        .clone()
}
