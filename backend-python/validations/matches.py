from typing import Optional
from datetime import datetime
from pydantic import BaseModel, Field


class ListMatcheQuery(BaseModel):
    limit: Optional[int] = Field(default=50, ge=1)

class CreateMatchPayload(BaseModel):
    sport: str = Field()
    home_team: str
    away_team: str
    start_time: datetime
    end_time: datetime
    home_score: Optional[int]
    away_score: Optional[int]

class UpdateScorePayload(BaseModel):
    home_score: int
    away_score: int
