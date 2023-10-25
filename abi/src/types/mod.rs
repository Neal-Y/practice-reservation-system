use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use sqlx::postgres::types::PgRange;

use crate::{utils::convert_time_to_utc, Error};

mod request;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_range() {
        let start = Timestamp {
            seconds: 0,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 1,
            nanos: 0,
        };
        assert!(validate_range(Some(&start), Some(&end)).is_ok());

        let start = Timestamp {
            seconds: 1,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 0,
            nanos: 0,
        };
        assert!(validate_range(Some(&start), Some(&end)).is_err());

        let start = Timestamp {
            seconds: 0,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 0,
            nanos: 0,
        };
        assert!(validate_range(Some(&start), Some(&end)).is_err());

        let start = Timestamp {
            seconds: 0,
            nanos: 1,
        };
        let end = Timestamp {
            seconds: 0,
            nanos: 0,
        };
        assert!(validate_range(Some(&start), Some(&end)).is_err());
    }

    #[test]
    fn test_convert_timestamp_into_timespan_pgrange() {
        let start = Timestamp {
            seconds: 0,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 1,
            nanos: 0,
        };
        let range = convert_timestamp_into_timespan_pgrange(Some(&start), Some(&end));
        assert_eq!(
            range.start,
            std::ops::Bound::Included(convert_time_to_utc(start))
        );
        assert_eq!(
            range.end,
            std::ops::Bound::Excluded(convert_time_to_utc(end))
        );
    }
}
