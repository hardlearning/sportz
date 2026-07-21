from dotenv import load_dotenv
load_dotenv()

from fastapi import FastAPI
from db import lifespan
from routes import match_router, commentary_router
from ws import ws_router


app = FastAPI(lifespan=lifespan)
app.include_router(match_router)
app.include_router(commentary_router)
app.include_router(ws_router)
