from typing import List, Optional

from pydantic import BaseModel, ConfigDict, Field

class ListCommentaryQuery(BaseModel):
    limit: Optional[int] = Field(default=10, gt=0, le=100)

class CreateCommentaryPayload(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    minute: Optional[int] = Field(default=None, ge=0)
    sequence: Optional[int] = Field(default=None, ge=0)
    period: Optional[str] = None
    event_type: Optional[str] = None
    actor: Optional[str] = None
    team: Optional[str] = None
    message: str = Field(min_length=1)
    metadata_json: Optional[dict] = Field(default=None, alias="metadata")
    tags: Optional[List[str]] = Field(default=None)