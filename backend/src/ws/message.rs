use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncomingMessage {
    Subscribe { match_id: i32 },
    Unsubscribe { match_id: i32 },
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutgoingMessage {
    Welcome,
    Subscribed { match_id: i32 },
    Unsubscribed { match_id: i32 },
    MatchCreated { data: serde_json::Value },
    ScoreUpdate { data: serde_json::Value },
    Commentary { data: serde_json::Value },
    Error { message: String }
}