use chrono::{DateTime, Utc};
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize)]
pub struct ListMatchQuery {
    pub limit: Option<u64>,
}

#[derive(Deserialize)]
pub struct CreateMatchPayload {
    pub sport: String,
    pub home_team: String,
    pub away_team: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateScorePayload {
    #[validate(range(min = 0))]
    pub home_score: i32,
    #[validate(range(min = 0))]
    pub away_score: i32,
}