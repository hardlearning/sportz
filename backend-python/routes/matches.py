from typing import Annotated
from fastapi import APIRouter, Query, Depends, HTTPException, status, Body
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from validations import ListMatcheQuery, CreateMatchPayload, UpdateScorePayload
from db import get_db, Match, MatchStatus
from utils import get_match_status, sync_match_status
from ws import broadcast_match_created, broadcast_score_update

match_router = APIRouter(prefix="/matches")

MAX_LIMIT = 100

@match_router.get("/")
async def list_matches(
    query: Annotated[ListMatcheQuery, Query()],
    db: Annotated[AsyncSession, Depends(get_db)]
):
    limit = min(query.limit or 50, MAX_LIMIT)

    try:
        sql = select(Match).order_by(Match.created_at).limit(limit)
        result = await db.execute(sql)
        matches = result.scalars().all()
        return { "data": matches }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": "Failed to list matches", "details": str(e)},
        )

@match_router.post("/", status_code=status.HTTP_201_CREATED)
async def create_match(
    payload: Annotated[CreateMatchPayload, Body()],
    db: Annotated[AsyncSession, Depends(get_db)]
):
    try:
        calculated_status = get_match_status(payload.start_time, payload.end_time)
        new_match = Match(
            sport=payload.sport,
            home_team=payload.home_team,
            away_team=payload.away_team,
            start_time=payload.start_time.replace(tzinfo=None),
            end_time=payload.end_time.replace(tzinfo=None),
            home_score=payload.home_score,
            away_score=payload.away_score,
            status=calculated_status,
        )
        db.add(new_match)
        await db.commit()
        await db.refresh(new_match)

        await broadcast_match_created(new_match.to_dict())

        return {"data": new_match}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": "Failed to create match", "details": str(e)},
        )

@match_router.patch("/{match_id}/score")
async def update_score(
    match_id: int,
    payload: Annotated[UpdateScorePayload, Body()],
    db: Annotated[AsyncSession, Depends(get_db)]
):
    try:
        stmt = select(Match).where(Match.id == match_id).limit(1)
        result = await db.execute(stmt)
        existing = result.scalar()
        if not existing:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail="Match not found",
            )
        async def status_updater(next_status):
            existing.status = next_status
            await db.commit()
        await sync_match_status(existing, status_updater)
        
        if existing.status != MatchStatus.LIVE:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT,
                detail="Match is not live",
            )
        
        # Mutate existing object tracked by the session (automatically updates)
        existing.home_score = payload.home_score
        existing.away_score = payload.away_score
        await db.commit()
        await db.refresh(existing)

        await broadcast_score_update(match_id, payload.home_score, payload.away_score)
        return { "data": existing }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": "Failed to update score", "details": str(e)},
        )
