use axum::Router;

use crate::common::state::SharedState;

mod matches;
mod commentary;

pub fn create_router() -> Router<SharedState> {
    let commentary_router = commentary::create_router();
    let match_router = matches::create_router().nest("/{match_id}/commentary", commentary_router);
    let api_router = Router::new().nest("/matches", match_router);
    Router::new().nest("/api", api_router)
}