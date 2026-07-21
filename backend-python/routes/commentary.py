from typing import Annotated
from fastapi import APIRouter, Body, Depends, HTTPException, Query, status
from sqlalchemy import desc, select
from validations import ListCommentaryQuery, CreateCommentaryPayload
from sqlalchemy.ext.asyncio import AsyncSession
from db import get_db, Commentary
from ws import broadcast_commentary

commentary_router = APIRouter(prefix="/matches/{match_id}/commentary")

MAX_LIMIT = 100

@commentary_router.get("/")
async def list_commentary(
    match_id: int,
    query: Annotated[ListCommentaryQuery, Query()],
    db: Annotated[AsyncSession, Depends(get_db)]
):
    safe_limit = min(query.limit or 10, MAX_LIMIT)
    try:
        sql = select(Commentary).where(Commentary.match_id == match_id).order_by(desc(Commentary.created_at)).limit(safe_limit)
        result = await db.execute(sql)
        commentaries = result.scalars().all()
        return {"data": commentaries}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": "Failed to fetch commentary", "details": str(e)},
        )

@commentary_router.post("/", status_code=status.HTTP_201_CREATED)
async def create_commentary(
    match_id: int,
    payload: Annotated[CreateCommentaryPayload, Body()],
    db: Annotated[AsyncSession, Depends(get_db)]
):
    try:
        new_commentary = Commentary(
            match_id=match_id,
            minute=payload.minute,
            sequence=payload.sequence,
            period=payload.period,
            event_type=payload.event_type,
            actor=payload.actor,
            team=payload.team,
            message=payload.message,
            metadata_json=payload.metadata_json,
            tags=payload.tags,
        )
        db.add(new_commentary)
        await db.commit()
        await db.refresh(new_commentary)

        await broadcast_commentary(match_id, new_commentary.to_dict())

        return {"data": new_commentary}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": "Failed to create commentary", "details": str(e)},
        )