use crate::graphql::utils::{
    get_crypto_service_from_ctx, get_http_client_from_ctx, get_user_repo_from_ctx,
};
use async_graphql::{Context, EmptySubscription, Error, ErrorExtensions, Object, Schema, Value};

use crate::config::crypto::CryptoService;
use crate::handlers::auth::GitHubEmailsResponse;
use validator::Validate;

use crate::models::user::{NewUser, SignInWithEmail, UpdateProfile, User, UserWithAuth};

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub struct Query;

#[Object]
impl Query {
    async fn user(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(min_length = 3))] email: String,
    ) -> Result<User, Error> {
        // get user repository
        let user_repo = get_user_repo_from_ctx(ctx);

        // find the user with a email
        let result = user_repo.find_by_email(email.clone()).await;

        match result {
            Ok(user) => Ok(user),
            Err(report) => {
                tracing::error!("Failed to fetch user: {:?}", report);
                Err(Error::new(format!(
                    "user with email `{}` does not exist",
                    email
                )))
            }
        }
    }

    async fn validate_access_token(&self, ctx: &Context<'_>, token: String) -> bool {
        let crypto_service = get_crypto_service_from_ctx(ctx);
        match crypto_service.decode_jwt(token) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

// TODO: Add mutation for `refresh_token`
pub struct Mutation;

#[Object]
impl Mutation {
    async fn sign_in_with_email(
        &self,
        ctx: &Context<'_>,
        input: SignInWithEmail,
    ) -> Result<UserWithAuth, Error> {
        let user_repo = get_user_repo_from_ctx(ctx);
        let crypto_service = get_crypto_service_from_ctx(ctx);

        let user = user_repo.find_by_email(input.email).await?;

        let password = match &user.password {
            Some(password) => password,
            None => return Err(Error::new("This user has no password")),
        };

        if crypto_service.hash_password(input.password).await? != String::from_utf8_lossy(password)
        {
            return Err(Error::new("invalid password"));
        }

        get_jwt_and_user(crypto_service, user)
    }

    async fn sign_in_with_github(
        &self,
        ctx: &Context<'_>,
        github_access_token: String,
    ) -> Result<UserWithAuth, Error> {
        let http_client = get_http_client_from_ctx(ctx);
        let user_repo = get_user_repo_from_ctx(ctx);
        let crypto_service = get_crypto_service_from_ctx(ctx);

        let res = http_client
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("token {}", github_access_token))
            .header("User-Agent", "curl/7.64.1")
            .send()
            .await?
            .json::<Vec<GitHubEmailsResponse>>()
            .await?;

        let email = res
            .into_iter()
            .find(|res| res.primary)
            .expect("unable to find a primary GitHub email")
            .email;

        let user = user_repo.find_by_email(email).await?;

        get_jwt_and_user(crypto_service, user)
    }

    async fn sign_up(&self, ctx: &Context<'_>, new_user: NewUser) -> Result<UserWithAuth, Error> {
        let user_repo = get_user_repo_from_ctx(ctx);
        let crypto_service = get_crypto_service_from_ctx(ctx);

        // validate the input
        match new_user.validate() {
            Ok(_) => (),
            Err(err) => {
                return Err(Error::new("error validating input").extend_with(|_, e| {
                    e.set(
                        "validation_error",
                        Value::from_json(serde_json::to_value(err.clone()).unwrap()).unwrap(),
                    )
                }))
            }
        };

        // create the user
        let user = user_repo.create(new_user.clone(), &crypto_service).await?;

        get_jwt_and_user(crypto_service, user)
    }

    async fn update_user(
        &self,
        ctx: &Context<'_>,
        update_profile: UpdateProfile,
        #[graphql(validator(min_length = 3))] username: String,
    ) -> Result<User, Error> {
        let user_repo = get_user_repo_from_ctx(ctx);
        let result = user_repo.update(update_profile, username.clone()).await;
        match result {
            Ok(user) => Ok(user),
            Err(report) => {
                tracing::error!("Failed to update user: {:?}", report);
                Err(Error::new(format!(
                    "unable to update user with username `{}`",
                    username
                )))
            }
        }
    }

    async fn delete_user(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(min_length = 3))] username: String,
    ) -> Result<User, Error> {
        let user_repo = get_user_repo_from_ctx(ctx);

        let result = user_repo.delete(username.clone()).await;
        match result {
            Ok(user) => Ok(user),
            Err(report) => {
                tracing::error!("Failed to delete user: {:?}", report);
                Err(Error::new(format!(
                    "user with username `{}` does not exist",
                    username
                )))
            }
        }
    }
}

fn get_jwt_and_user(crypto_service: CryptoService, user: User) -> Result<UserWithAuth, Error> {
    crypto_service
        .generate_jwt(String::from_utf8_lossy(&user.email).to_string())
        .map(|claim| UserWithAuth {
            user,
            access_token: claim.0,
            expired_at: claim.1,
        })
        .map_err(|err| {
            Error::new("error generating jwt")
                .extend_with(|_, e| e.set("jwt_error", err.to_string()))
        })
}
