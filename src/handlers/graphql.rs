use actix_web::{get, post, web, HttpResponse, Responder};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::GraphQLRequest;

use crate::graphql::schema::AppSchema;

#[post("/graphql")]
pub(crate) async fn graphql(
    schema: web::Data<AppSchema>,
    request: GraphQLRequest,
) -> impl Responder {
    let schema = schema.into_inner();
    let res = schema.execute(request.into_inner()).await;
    HttpResponse::Ok().json(res)
}

#[get("/playground")]
pub(crate) async fn playground() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}
