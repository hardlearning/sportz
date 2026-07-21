from json import JSONDecodeError
from typing import Set
from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from .connection import ConnectionManager

ws_router = APIRouter()

ws_manager = ConnectionManager()

async def broadcast_match_created(match: dict):
    await ws_manager.broadcast_to_all({"type": "match_created", "data": match})

async def broadcast_score_update(match_id: int, home_score: int, away_score: int):
    await ws_manager.broadcast_to_match(match_id, {"type": "score_update", "data": {
        "home_score": home_score,
        "away_score": away_score,
    }})

async def broadcast_commentary(match_id: int, comment: dict):
    await ws_manager.broadcast_to_match(match_id, {"type": "commentary", "data": comment})

@ws_router.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await ws_manager.connect(websocket)
    local_subscriptions: Set[int] = set()
    try:
        await ws_manager.send_personal_message(websocket, {"type": "welcome"})
        while True:
            try:
                message = await websocket.receive_json()
            except JSONDecodeError:
                await ws_manager.send_personal_message(websocket, {"type": "error", "message": "Invalid JSON"})
                continue
            msg_type = message.get("type")
            match_id = message.get("match_id")
            if msg_type == "subscribe" and isinstance(match_id, int):
                ws_manager.subscribe(match_id, websocket)
                local_subscriptions.add(match_id)
                await ws_manager.send_personal_message(websocket, {"type": "subscribed", "match_id": match_id})
            elif msg_type == "unsubscribe" and isinstance(match_id, int):
                ws_manager.unsubscribe(match_id, websocket)
                local_subscriptions.discard(match_id)
                await ws_manager.send_personal_message(websocket, {"type": "unsubscribed", "match_id": match_id})
            else:
                await ws_manager.send_personal_message(websocket, {"type": "error", "message": "Invalid type or match_id"})
    except WebSocketDisconnect:
        ws_manager.disconnect(websocket)
        ws_manager.cleanup_subscriptions(websocket, local_subscriptions)