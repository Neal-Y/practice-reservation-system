use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;

pub fn convert_time_to_utc(ts: &Timestamp) -> DateTime<Utc> {
    let naive_dt =
        NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as _).expect("Invalid timestamp");
    DateTime::<Utc>::from_utc(naive_dt, Utc)
}

pub fn convert_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as _,
    }
}
