pub mod auth;
pub mod graphql;
pub mod health;
pub mod user;

use actix_web::web::ServiceConfig;

pub fn app_config(config: &mut ServiceConfig) {
    config.service(health::health);
    config.service(user::find);
    config.service(user::create);
    config.service(user::delete);
    config.service(user::sign_in_with_email);
    config.service(auth::redirect);
    config.service(auth::test_login);
    config.service(graphql::graphql);
    config.service(graphql::playground);
}
