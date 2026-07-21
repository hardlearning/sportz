from datetime import datetime, timezone
from typing import Awaitable, Callable, Optional
from db import MatchStatus, Match

def get_match_status(
    start_time: Optional[datetime],
    end_time: Optional[datetime],
    now: Optional[datetime] = None
) -> Optional[MatchStatus]:
    """
    Determines the current match status based on time boundaries.
    """
    if now is None:
        now = datetime.now()
    if not start_time or not end_time:
        return None
    start_time = start_time.replace(tzinfo=None)
    if now < start_time:
        return MatchStatus.SCHEDULED
    end_time = end_time.replace(tzinfo=None)
    if now >= end_time:
        return MatchStatus.FINISHED
    return MatchStatus.LIVE

async def sync_match_status(
    match: Match,
    update_status: Callable[[MatchStatus], Awaitable[None]]
) -> MatchStatus:
    next_status = get_match_status(match.start_time, match.end_time)
    if not next_status:
        return match.status
    if match.status != next_status:
        await update_status(next_status)
        match.status = next_status
    return match.status