use std::collections::HashMap;
use tokio::sync::{RwLock, broadcast, mpsc};
use crate::ws::message::OutgoingMessage;

pub struct ConnectionManager {
    match_subscribers: RwLock<HashMap<i32, Vec<mpsc::Sender<OutgoingMessage>>>>,
    global_broadcast: broadcast::Sender<OutgoingMessage>
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (global_broadcast, _) = broadcast::channel(100);
        Self {
            match_subscribers: RwLock::new(HashMap::new()),
            global_broadcast
        }
    }

    pub fn subscribe_broadcast(&self) -> broadcast::Receiver<OutgoingMessage> {
        self.global_broadcast.subscribe()
    }

    pub async fn subscribe(&self, match_id: i32, client_tx: mpsc::Sender<OutgoingMessage>) {
        let mut subs = self.match_subscribers.write().await;
        subs.entry(match_id).or_insert_with(Vec::new).push(client_tx);
    }

    pub async fn unsubscribe(&self, match_id: i32, client_tx: &mpsc::Sender<OutgoingMessage>) {
        let mut subs = self.match_subscribers.write().await;
        if let Some(senders) = subs.get_mut(&match_id) {
            senders.retain(|tx| !tx.same_channel(client_tx));
            if senders.is_empty() {
                subs.remove(&match_id);
            }
        }
    }

    pub fn broadcast_match_created(&self, match_data: serde_json::Value) {
        let _ = self.global_broadcast.send(OutgoingMessage::MatchCreated { data: match_data });
    }

    pub async fn broadcast_score_update(&self, match_id: i32, home_score: i32, away_score: i32) {
        let subs = self.match_subscribers.read().await;
        if let Some(senders) = subs.get(&match_id) {
            let message = OutgoingMessage::ScoreUpdate { data: serde_json::json!({
                "home_score": home_score,
                "away_score": away_score
            }) };
            // Loop through all subscribed client channels and deliver the payload
            for tx in senders {
                let _ = tx.send(message.clone()).await;
            }
        }
    }

    pub async fn broadcast_commentary(&self, match_id: i32, comment: serde_json::Value) {
        let subs = self.match_subscribers.read().await;
        if let Some(senders) = subs.get(&match_id) {
            let message = OutgoingMessage::Commentary { data: comment };
            // Loop through all subscribed client channels and deliver the payload
            for tx in senders {
                let _ = tx.send(message.clone()).await;
            }
        }
    }
}