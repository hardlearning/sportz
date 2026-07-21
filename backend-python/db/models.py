import enum
from datetime import datetime
from typing import List
from typing_extensions import Annotated
from sqlalchemy import ARRAY, DateTime, ForeignKey, Integer, String, Enum, text
from sqlalchemy.dialects.postgresql import JSONB
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship


class Base(DeclarativeBase):
    __abstract__ = True  # Ensures SQLAlchemy doesn't try to create a table for Base

    def to_dict(self):
        # Extracts all mapped column names and their current values
        return {attr.key: getattr(self, attr.key) for attr in self.__mapper__.column_attrs}

intpk = Annotated[int, mapped_column(primary_key=True, autoincrement=True)]

# Define the Python enum for match status
class MatchStatus(str, enum.Enum):
    SCHEDULED = "scheduled"
    LIVE = "live"
    FINISHED = "finished"

class Match(Base):
    __tablename__ = "matches"

    id: Mapped[intpk]
    sport: Mapped[str] = mapped_column(String, nullable=False)
    home_team: Mapped[str] = mapped_column(String, nullable=False)
    away_team: Mapped[str] = mapped_column(String, nullable=False)

    # Uses PostgreSQL native ENUM type
    status: Mapped[MatchStatus] = mapped_column(
        Enum(MatchStatus),
        nullable=False,
        default=MatchStatus.SCHEDULED,
    )

    start_time: Mapped[datetime] = mapped_column(DateTime, nullable=True)
    end_time: Mapped[datetime] = mapped_column(DateTime, nullable=True)
    home_score: Mapped[int] = mapped_column(Integer, nullable=False, default=0)
    away_score: Mapped[int] = mapped_column(Integer, nullable=False, default=0)

    # Server-side default for NOW()
    created_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=text("CURRENT_TIMESTAMP")
    )

    # Optional: Relationship helper to access commentaries from a match object
    commentaries: Mapped[List["Commentary"]] = relationship("Commentary", back_populates="match")


class Commentary(Base):
    __tablename__ = "commentary"

    id: Mapped[intpk]
    match_id = mapped_column(
        Integer, ForeignKey("matches.id"), nullable=False
    )

    minute: Mapped[int] = mapped_column(Integer, nullable=True)
    sequence: Mapped[int] = mapped_column(Integer, nullable=True)
    period: Mapped[str] = mapped_column(String, nullable=True)
    event_type: Mapped[str] = mapped_column(String, nullable=True)
    actor: Mapped[str] = mapped_column(String, nullable=True)
    team: Mapped[str] = mapped_column(String, nullable=True)
    message: Mapped[str] = mapped_column(String, nullable=False)

    # PostgreSQL specific types
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=True)  # 'metadata' is a reserved keyword in SQLAlchemy
    tags: Mapped[list] = mapped_column(ARRAY(String), nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=text("CURRENT_TIMESTAMP")
    )

    # Optional: Relationship helper to access the parent match object
    match: Mapped["Match"] = relationship("Match", back_populates="commentaries")
