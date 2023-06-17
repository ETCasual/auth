use crate::AppState;
use actix_web::{get, web, HttpResponse, Responder};

#[get("/health")]
pub(crate) async fn health(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!("{} is healthy!", data.app_name))
}

#[cfg(test)]
mod tests {
    use crate::handlers::app_config;
    use crate::AppState;
    use actix_web::{test, web, App};

    #[actix_rt::test]
    async fn test_health_ok() {
        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    app_name: String::from("Auth"),
                    app_config: todo!(),
                }))
                .configure(app_config),
        )
        .await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "Healthcheck");
    }
}
