use crate::db::user::UserRepository;
use crate::models::user::{NewUser, User, UserWithAuth};
use crate::{config::crypto::CryptoService, models::user::SignInWithEmail};
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use validator::Validate;

#[get("/user/{email}")]
pub(crate) async fn find(
    user_repo: web::Data<UserRepository>,
    email: web::Path<String>,
) -> impl Responder {
    let email = email.into_inner();
    let result = user_repo.get_ref().find_by_email(email).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        _ => HttpResponse::BadRequest().body("Failed to find user"),
    }
}

#[post("/user")]
pub(crate) async fn create(
    user_repo: web::Data<UserRepository>,
    crypto_service: web::Data<CryptoService>,
    new_user: web::Json<NewUser>,
) -> impl Responder {
    // unwrap the param
    let new_user = new_user.into_inner();
    // validate the new_user input
    match new_user.validate() {
        Ok(_) => (),
        Err(e) => return HttpResponse::BadRequest().json(e),
    };
    // create the user
    let result = user_repo.get_ref().create(new_user, &crypto_service).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => {
            tracing::error!("Failed to create user: {}", e);
            return HttpResponse::BadRequest().body("Failed to create user");
        }
    }
}

#[delete("/user/{email}")]
pub(crate) async fn delete(
    user_repo: web::Data<UserRepository>,
    email: web::Path<String>,
) -> impl Responder {
    let email = email.into_inner();
    // delete the user
    let result = user_repo.get_ref().delete(email).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        _ => HttpResponse::BadRequest().body("Failed to delete user"),
    }
}

#[post("/user/signIn")]
pub(crate) async fn sign_in_with_email(
    user_repo: web::Data<UserRepository>,
    crypto_service: web::Data<CryptoService>,
    credentials: web::Json<SignInWithEmail>,
) -> impl Responder {
    let user = match user_repo.find_by_email(credentials.email.clone()).await {
        Ok(user) => user,
        Err(e) => return HttpResponse::BadRequest().body(format!("Failed to find user: {}", e)),
    };

    let password = match &user.password {
        Some(password) => password,
        None => return HttpResponse::BadRequest().body("This user has no password"),
    };

    let hash = match crypto_service
        .hash_password(credentials.password.clone())
        .await
    {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to hash password: {}", e))
        }
    };

    if hash != String::from_utf8_lossy(password) {
        return HttpResponse::BadRequest().body("invalid password");
    }

    get_jwt_and_user(crypto_service, user)
}

fn get_jwt_and_user(crypto_service: web::Data<CryptoService>, user: User) -> HttpResponse {
    match crypto_service.generate_jwt(String::from_utf8_lossy(&user.email).to_string()) {
        Ok(claim) => HttpResponse::Ok().json(UserWithAuth {
            user,
            access_token: claim.0,
            expired_at: claim.1,
        }),
        Err(_) => HttpResponse::InternalServerError().body("error generating jwt"),
    }
}

#[cfg(test)]
mod tests {
    use crate::handlers::app_config;
    use crate::models::user::User;
    use crate::Config;
    use crate::UserRepository;
    use actix_web::{test, web, App};
    use serde_json::json;
    use std::sync::Arc;

    #[actix_rt::test]
    async fn test_user() {
        let config: Config = Config::from_env(false).expect("Server configuration");

        let pool = config.new_db_pool().await.expect("Database configuration");

        let crypto_service = config.new_crypto_service();

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(web::Data::new(crypto_service.clone()))
                .app_data(web::Data::new(UserRepository::new(Arc::new(pool.clone()))))
                .configure(app_config),
        )
        .await;

        // the test user's username
        let test_username = String::from("test_user");

        // request body for creating a user
        let req_body = json!({
            "username": test_username,
            "email": "test@test.com",
            "password": "test1234"
        });

        // creating a user
        let resp = test::TestRequest::post()
            .uri("/user")
            .set_json(&req_body)
            .send_request(&mut app)
            .await;
        assert!(resp.status().is_success(), "Failed to create user");

        // creating an existing user
        let resp = test::TestRequest::post()
            .uri("/user")
            .set_json(&req_body)
            .send_request(&mut app)
            .await;
        assert!(
            resp.status().is_client_error(),
            "Should not be possible to create user with same username"
        );

        // finding a user
        let resp = test::TestRequest::get()
            .uri(&format!("/user/{}", test_username))
            .send_request(&mut app)
            .await;
        assert!(resp.status().is_success(), "Failed to find user");

        // parse the result as a user
        let test_user: User = test::read_body_json(resp).await;
        assert_eq!(
            String::from_utf8_lossy(&test_user.email),
            test_username,
            "Found wrong user"
        );

        // deleting a user
        let resp = test::TestRequest::delete()
            .uri(&format!(
                "/user/{}",
                String::from_utf8_lossy(&test_user.email)
            ))
            .send_request(&mut app)
            .await;
        assert!(resp.status().is_success(), "Failed to delete user");
    }
}

// curl -v -X POST -d '{"email": "casualorient@gmail.com", "password": "123456abc"}' -H "Content-Type: application/json" localhost:8080/user
// curl -v -X POST -d '{"email": "casualorient@gmail.com", "password": "123456abc"}' -H "Content-Type: application/json" localhost:8080/user/signIn
