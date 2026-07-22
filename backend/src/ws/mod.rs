use axum::{Router, routing::any};
use crate::common::state::SharedState;

mod handler;
pub mod connection;
mod message;

pub fn create_router() -> Router<SharedState> {
    Router::new()
        .route("/ws", any(handler::ws_handler))
}
