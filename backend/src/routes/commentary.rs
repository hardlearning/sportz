use axum::{Json, Router, extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, routing::get};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::{common::{constants::MAX_LIMIT, response::{ErrorResponse, GenericResponse}, state::SharedState}, entity::commentary, validation::commentary::{CreateCommentaryPayload, ListCommentaryQuery}};
use crate::entity::prelude::Commentary;

pub fn create_router() -> Router<SharedState> {
    Router::new().route("/", get(list_commentaries).post(create_commentary))
}

async fn list_commentaries(
    Path(match_id): Path<i32>,
    Query(query): Query<ListCommentaryQuery>,
    State(state): State<SharedState>
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(10).min(MAX_LIMIT);

    let results = Commentary::find()
        .filter(commentary::Column::MatchId.eq(match_id))
        .order_by_desc(commentary::Column::CreatedAt)
        .limit(limit)
        .all(&state.db)
        .await;

    match results {
        Ok(commentaries) => (
            StatusCode::OK,
            Json(GenericResponse {
                data: commentaries,
            }),
        ).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch commentary".into(),
                details: None,
            }),
        ).into_response()
    }
}

async fn create_commentary(
    Path(match_id): Path<i32>,
    State(state): State<SharedState>,
    Json(payload): Json<CreateCommentaryPayload>
) -> impl IntoResponse {
    let mut new_commentary: commentary::ActiveModel = payload.into();
    new_commentary.match_id = Set(match_id);

    match new_commentary.insert(&state.db).await {
        Ok(inserted) => {
            let json_comment = serde_json::to_value(inserted.clone()).unwrap();
            state.connection.broadcast_commentary(match_id, json_comment).await;
            (
                StatusCode::OK,
                Json(GenericResponse {
                    data: inserted
                })
            ).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create commentary".into(),
                details: Some(e.to_string()),
            }),
        ).into_response()
    }
}