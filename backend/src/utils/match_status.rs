use chrono::{DateTime, Utc};

use crate::entity::matches;
use crate::entity::sea_orm_active_enums::MatchStatus;

pub fn get_match_status(start: DateTime<Utc>, end: DateTime<Utc>) -> MatchStatus {
    let now = Utc::now();
    if now < start {
        MatchStatus::Scheduled
    } else if now > end {
        MatchStatus::Finished
    } else {
        MatchStatus::Live
    }
}

pub fn sync_match_status<F>(existing: &matches::Model, update_fn: F) 
where 
    F: FnOnce(MatchStatus) {
    let start_time = existing.start_time.and_utc();
    let end_time = existing.end_time.and_utc();
    let next_status = get_match_status(start_time, end_time);
    if next_status != existing.status {
        update_fn(next_status);
    }
}