mod model;

use async_graphql::http::playground_source;
use async_graphql::{EmptySubscription, Schema};
use sqlx::postgres::PgPool;
use tide::{
    http::{headers, mime},
    Request, Response, StatusCode,
};

struct AppState {
    schema: Schema<model::QueryRoot, model::MutationRoot, EmptySubscription>,
}

pub struct GraphQLContext {
    pub pool: PgPool,
}

impl GraphQLContext {
    fn new(pool: PgPool) -> Self {
        GraphQLContext { pool }
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = PgPool::builder()
        .max_size(5)
        .build("postgresql://postgres:postgres@localhost:5433/crud_test")
        .await?;

    let listen_addr = "localhost:8080";
    let schema = Schema::build(model::QueryRoot, model::MutationRoot, EmptySubscription)
        .data(GraphQLContext::new(pool))
        .finish();

    println!("I will start playground at http://{}", listen_addr);

    let app_state = AppState { schema };
    let mut app = tide::with_state(app_state);

    async fn graphql(req: Request<AppState>) -> tide::Result<Response> {
        let schema = req.state().schema.clone();
        async_graphql_tide::graphql(req, schema, |query_builder| query_builder).await
    }

    app.at("/graphql").post(graphql).get(graphql);

    app.at("/").get(|_| async move {
        let resp = Response::new(StatusCode::Ok)
            .body_string(playground_source("/graphql", None))
            .set_header(headers::CONTENT_TYPE, mime::HTML.to_string());
        Ok(resp)
    });

    app.listen(listen_addr).await?;

    Ok(())
}
