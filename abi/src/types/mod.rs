use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use sqlx::postgres::types::PgRange;

use crate::{utils::convert_time_to_utc, Error};

mod reservation;
mod reservation_query;
mod reservation_status;

pub fn validate_range(start: Option<&Timestamp>, end: Option<&Timestamp>) -> Result<(), Error> {
    // check the timestamp, can not be empty
    if start.is_none() || end.is_none() {
        return Err(Error::InvalidTime);
    }

    let start = start.as_ref().unwrap();
    let end = end.as_ref().unwrap();

    // Timestamp中的結構問題，如果是start.seconds > end.seconds，那麼就是時間錯誤。但由於有可能出現秒數一樣的狀況，所以要再加上start.nanos >= end.nanos
    if start.seconds > end.seconds || (start.seconds == end.seconds && start.nanos >= end.nanos) {
        return Err(Error::InvalidTime);
    }
    Ok(())
}

pub fn convert_timestamp_into_timespan_pgrange(
    start: Option<&Timestamp>,
    end: Option<&Timestamp>,
) -> PgRange<DateTime<Utc>> {
    let start = convert_time_to_utc(start.unwrap().clone());
    let end = convert_time_to_utc(end.unwrap().clone());
    PgRange {
        start: std::ops::Bound::Included(start),
        end: std::ops::Bound::Excluded(end),
    }
}
