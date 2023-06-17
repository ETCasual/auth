use crate::models::user::UserWithAuth;
use crate::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub(crate) struct GitHubRedirectParams {
    code: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitHubAccessTokenResponse {
    access_token: String,
    scope: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitHubEmailsResponse {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
    pub visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse {
    data: SignInWithGithub,
}

#[derive(Debug, Deserialize)]
struct SignInWithGithub {
    sign_in_with_github: UserWithAuth,
}

#[get("/oauth/github")]
pub(crate) async fn redirect(
    params: web::Query<GitHubRedirectParams>,
    http_client: web::Data<reqwest::Client>,
    state: web::Data<AppState>,
) -> impl Responder {
    let res = http_client
        .post(format!(
            "https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
            state.app_config.github_client_id, state.app_config.github_client_secret, params.code
        ))
        .header("accept", "application/json")
        .send()
        .await
        .expect("unable to send request to GitHub OAuth")
        .json::<GitHubAccessTokenResponse>()
        .await
        .expect("unable to deserialize response from GitHub OAuth");

    let mut body = HashMap::new();
    body.insert(
        "query",
        format!(
            "
            mutation {{
                sign_in_with_github(githubAccessToken: \"{}\") {{
                    access_token: accessToken
                    expired_at: expiredAt
                    user {{
                        id
                        username
                        email
                        full_name: fullName
                        bio
                        avatar_url: avatarUrl
                        created_at: createdAt
                        updated_at: updatedAt
                    }}
                }}
            }}
            ",
            res.access_token
        ),
    );

    let user = http_client
        .post("http://localhost:8080/graphql")
        .json(&body)
        .send()
        .await
        .expect("unable to send request to Auth API")
        .json::<GraphQLResponse>()
        .await
        .expect("unable to deserialize response from Auth API");

    println!("{:?}", user);

    HttpResponse::Found()
        .append_header((
            "Location",
            format!(
                "/health?access_token={}",
                user.data.sign_in_with_github.access_token
            ),
        ))
        .finish()
}

#[get("/test_login")]
pub(crate) async fn test_login(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(r#"
            <!DOCTYPE html>
            <html>
              <body>
                <a
                  href="https://github.com/login/oauth/authorize?client_id={}&redirect_uri=http://localhost:8080/oauth/github"
                >
                  Login with github
                </a>
              </body>
            </html>
        "#, state.app_config.github_client_id))
}
