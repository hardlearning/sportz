use axum::{Json, Router, extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, routing::{get, patch}};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, QueryOrder, QuerySelect};

use crate::{common::{constants::MAX_LIMIT, response::{ErrorResponse, GenericResponse}, state::SharedState}, entity::{matches, sea_orm_active_enums::MatchStatus}, utils::match_status::{get_match_status, sync_match_status}, validation::matches::{CreateMatchPayload, ListMatchQuery, UpdateScorePayload}};
use crate::entity::prelude::Matches;

pub fn create_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(list_matches).post(create_match))
        .route("/{id}/score", patch(update_match_score))
}

#[axum::debug_handler]
async fn list_matches(
    Query(query): Query<ListMatchQuery>,
    State(state): State<SharedState>
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).min(MAX_LIMIT);

    let result = Matches::find()
        .order_by_desc(matches::Column::CreatedAt)
        .limit(limit)
        .all(&state.db).await;

    match result {
        Ok(matches) => (StatusCode::OK, Json(GenericResponse { data: matches })).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to list matches.".into(),
                details: None
            })
        ).into_response()
    } 
}

#[axum::debug_handler]
async fn create_match(
    State(state): State<SharedState>,
    Json(payload): Json<CreateMatchPayload>,
) -> impl IntoResponse {
    let status = get_match_status(payload.start_time, payload.end_time);
    let new_match = matches::ActiveModel {
        sport: Set(payload.sport),
        home_team: Set(payload.home_team),
        away_team: Set(payload.away_team),
        home_score: Set(payload.home_score.unwrap_or(0)),
        away_score: Set(payload.away_score.unwrap_or(0)),
        start_time: Set(payload.start_time.naive_utc()),
        end_time: Set(payload.end_time.naive_utc()),
        status: Set(status),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    match new_match.insert(&state.db).await {
        Ok(inserted) => {
            let json_msg = serde_json::to_value(inserted.clone()).unwrap();
            state.connection.broadcast_match_created(json_msg);

            (
                StatusCode::CREATED,
                Json(GenericResponse { data: inserted }),
            ).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create match.".into(),
                details: Some(e.to_string()),
            }),
        ).into_response()
    }
}

#[axum::debug_handler]
async fn update_match_score(
    Path(match_id): Path<i32>,
    State(state): State<SharedState>,
    Json(payload): Json<UpdateScorePayload>,
) -> impl IntoResponse {
    // 1. Locate existing match
    let match_lookup = Matches::find_by_id(match_id).one(&state.db).await;

    let mut existing_match = match match_lookup {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Match not found".into(),
                    details: None,
                }),
            ).into_response()
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update score".into(),
                    details: None,
                }),
            ).into_response()
        }
    };

    // 2. Synchronize match status dynamically
    let mut status_changed = false;
    let mut fresh_status = existing_match.status.clone();

    sync_match_status(&existing_match, |next_status| {
        status_changed = true;
        fresh_status = next_status;
    });

    if status_changed {
        let mut active: matches::ActiveModel = existing_match.clone().into();
        active.status = Set(fresh_status.clone());
        if let Ok(updated_model) = active.update(&state.db).await {
            existing_match = updated_model;
        }
    }

    // 3. Confirm match is currently Live
    if fresh_status != MatchStatus::Live {
        return (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Match is not live".into(),
                details: None,
            }),
        ).into_response();
    }

    // 4. Update the scores
    let mut active: matches::ActiveModel = existing_match.into();
    active.home_score = Set(payload.home_score);
    active.away_score = Set(payload.away_score);

    match active.update(&state.db).await {
        Ok(updated) => {
            // Call broadcast logic from the WebSocket loop here:
            state.connection.broadcast_score_update(match_id, updated.home_score, updated.away_score).await;
            
            (StatusCode::OK, Json(GenericResponse { data: updated })).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to update score".into(),
                details: None,
            }),
        ).into_response()
    }
}