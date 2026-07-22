use std::sync::Arc;
use tracing::info;
use axum::Router;
use common::state::AppState;

mod common;
mod routes;
mod ws;
mod validation;
mod entity;
mod utils;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    common::logger::init();

    let db = common::database::init().await.expect("Database connection failed!");
    let shared_state = Arc::new(AppState::new(db));

    let rest_router = routes::create_router();
    let ws_router = ws::create_router();
    let app = Router::new()
        .merge(rest_router)
        .merge(ws_router)
        .with_state(shared_state);

    let addr = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server running on {addr}");

    axum::serve(listener, app).await
}
