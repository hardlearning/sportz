use sea_orm::ActiveValue::Set;
use serde::Deserialize;

use crate::entity::commentary;

#[derive(Deserialize)]
pub struct ListCommentaryQuery {
    pub limit: Option<u64>
}

#[derive(Deserialize)]
pub struct CreateCommentaryPayload {
    pub minute: Option<i32>,
    pub sequence: Option<i32>,
    pub period: Option<String>,
    pub event_type: Option<String>,
    pub actor: Option<String>,
    pub team: Option<String>,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
}

impl From<CreateCommentaryPayload> for commentary::ActiveModel {
    fn from(payload: CreateCommentaryPayload) -> Self {
        Self {
            minute: Set(payload.minute),
            sequence: Set(payload.sequence),
            period: Set(payload.period),
            event_type: Set(payload.event_type),
            actor: Set(payload.actor),
            team: Set(payload.team),
            message: Set(payload.message),
            metadata: Set(payload.metadata),
            tags: Set(payload.tags),
            ..Default::default()
        }
    }
}
