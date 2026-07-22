use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}}, response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use crate::{common::state::AppState, ws::message::{IncomingMessage, OutgoingMessage}};

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let mut local_subscriptions = HashSet::new();

    let (client_tx, mut client_rx) = mpsc::channel(50);

    if let Err(err) = handle_loop(socket, state.clone(), &client_tx, &mut client_rx, &mut local_subscriptions).await {
        error!("handle_socket error: {}", err);
    }
    info!("Break out of the loop");

    for match_id in local_subscriptions {
        state.connection.unsubscribe(match_id, &client_tx).await;
    }
}

async fn handle_loop(
    socket: WebSocket,
    state: Arc<AppState>,
    client_tx: &mpsc::Sender<OutgoingMessage>,
    client_rx: &mut mpsc::Receiver<OutgoingMessage>,
    local_subscriptions: &mut HashSet<i32>
) -> anyhow::Result<()> {
    let (mut sender, mut receiver) = socket.split();

    let mut global_rx = state.connection.subscribe_broadcast();

    let welcome = serde_json::to_string(&OutgoingMessage::Welcome)?;
    sender.send(Message::Text(welcome.into())).await?;

    loop {
        tokio::select! {
            Some(result) = receiver.next() => {
                match result? {
                    Message::Text(message) => {
                        let out_msg = handle_text_message(&message, client_tx.clone(), state.clone(), local_subscriptions).await?;
                        let text_msg = serde_json::to_string(&out_msg)?;
                        if sender.send(Message::Text(text_msg.into())).await.is_err() {
                            break;
                        }
                    }
                    Message::Binary(_) => warn!("Received binary message"),
                    Message::Ping(_) => warn!("Received ping message"),
                    Message::Pong(_) => warn!("Received pong message"),
                    Message::Close(_) => {
                        warn!("Received close message");
                        break;
                    }
                }
            }
            Some(msg) = client_rx.recv() => {
                let text = serde_json::to_string(&msg)?;
                if sender.send(Message::Text(text.into())).await.is_err() { break; }
            }
            Ok(msg) = global_rx.recv() => {
                let text = serde_json::to_string(&msg)?;
                if sender.send(Message::Text(text.into())).await.is_err() { break; }
            }
        }
    }
    Ok(())
}

async fn handle_text_message(message: &str, client_tx: mpsc::Sender<OutgoingMessage>, state: Arc<AppState>, local_subscriptions: &mut HashSet<i32>) -> anyhow::Result<OutgoingMessage> {
    let out_msg = match serde_json::from_str(message) {
        Ok(IncomingMessage::Subscribe { match_id }) => {
            if local_subscriptions.insert(match_id) {
                state.connection.subscribe(match_id, client_tx).await;
            }
            OutgoingMessage::Subscribed { match_id }
        }
        Ok(IncomingMessage::Unsubscribe { match_id }) => {
            if local_subscriptions.remove(&match_id) {
                state.connection.unsubscribe(match_id, &client_tx).await;
            }
            OutgoingMessage::Unsubscribed { match_id }
        }
        Err(_) => {
            OutgoingMessage::Error { message: "Invalid JSON".into() }
        }
    };
    Ok(out_msg)
}