import os
from typing import AsyncGenerator
from contextlib import asynccontextmanager
from fastapi import FastAPI
from sqlalchemy.ext.asyncio import create_async_engine, async_sessionmaker, AsyncSession
from db.models import Base

db_url = os.getenv("DATABASE_URL")
assert db_url != None
engine = create_async_engine(db_url, echo=True)
DatabaseSession = async_sessionmaker(engine)

async def get_db() -> AsyncGenerator[AsyncSession, None]:
    async with DatabaseSession() as session:
        yield session

# Automatically create tables on startup
@asynccontextmanager
async def lifespan(app: FastAPI):
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
    yield
    await engine.dispose()
