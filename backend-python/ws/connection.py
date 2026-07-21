import asyncio
import json
from typing import Dict, Set
from fastapi import WebSocket
from utils import DateTimeEncoder


class ConnectionManager:
    def __init__(self):
        self.active_connections: Set[WebSocket] = set()
        self.match_subscribers: Dict[int, Set[WebSocket]] = {}

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.active_connections.add(websocket)

    def disconnect(self, websocket: WebSocket):
        self.active_connections.discard(websocket)

    async def send_personal_message(self, websocket: WebSocket, message: dict):
        await websocket.send_json(message)

    async def broadcast_to_all(self, message: dict):
        if not self.active_connections:
            return
        json_string = json.dumps(message, cls=DateTimeEncoder)
        await asyncio.gather(
            *(client.send_text(json_string) for client in self.active_connections),
            return_exceptions=True
        )

    async def broadcast_to_match(self, match_id: int, message: dict):
        subscribers = self.match_subscribers.get(match_id)
        if not subscribers:
            return
        json_string = json.dumps(message, cls=DateTimeEncoder)
        await asyncio.gather(
            *(client.send_text(json_string) for client in subscribers),
            return_exceptions=True
        )
    
    def subscribe(self, match_id: int, websocket: WebSocket):
        if match_id not in self.match_subscribers:
            self.match_subscribers[match_id] = {websocket}
        self.match_subscribers[match_id].add(websocket)
    
    def unsubscribe(self, match_id: int, websocket: WebSocket):
        if match_id in self.match_subscribers:
            self.match_subscribers[match_id].discard(websocket)
            if not self.match_subscribers[match_id]:
                del self.match_subscribers[match_id]
    
    def cleanup_subscriptions(self, websocket: WebSocket, subscriptions: Set[int]):
        for match_id in list(subscriptions):
            self.unsubscribe(match_id, websocket)