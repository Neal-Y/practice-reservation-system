use chrono::{DateTime, FixedOffset, Utc};
use sqlx::{
    postgres::{types::PgRange, PgRow},
    FromRow, Row,
};
use std::ops::Bound;

use crate::{
    convert_timestamp_into_timespan_pgrange, types::reservation_status::RsvpStatus,
    utils::convert_to_timestamp, validate_range, Error, Reservation, ReservationStatus, Validator,
};

impl Reservation {
    pub fn new_pending(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            id: 0,
            user_id: uid.into(),
            status: ReservationStatus::Pending as i32,
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: note.into(),
        }
    }

    pub fn get_timestamp(&self) -> PgRange<DateTime<Utc>> {
        convert_timestamp_into_timespan_pgrange(
            Some(self.start.as_ref().unwrap()),
            Some(self.end.as_ref().unwrap()),
        )
    }
}

// to make sure change_status() works, we need to implement FromRow trait for Reservation
impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let range: PgRange<DateTime<Utc>> = row.get("timespan");
        let range: NativeRange<DateTime<Utc>> = range.into();

        let start = range.start.unwrap();
        let end = range.end.unwrap();

        Ok(Self {
            id: row.get::<i64, _>("id"),
            user_id: row.get("user_id"),
            resource_id: row.get("resource_id"),
            start: Some(convert_to_timestamp(start)),
            end: Some(convert_to_timestamp(end)),
            note: row.get("note"),
            status: ReservationStatus::from(row.get::<RsvpStatus, _>("status")) as i32,
        })
    }
}

// to make sure we can divided the "timespan" to get start and end side by side.
struct NativeRange<T> {
    start: Option<T>,
    end: Option<T>,
}

// if we want to unwrap the PgRange, we have to match the Bound enum to make sure that every branch is considered.

impl<T> From<PgRange<T>> for NativeRange<T> {
    fn from(range: PgRange<T>) -> Self {
        let f = |b: Bound<T>| match b {
            Bound::Included(t) => Some(t),
            Bound::Excluded(t) => Some(t),
            Bound::Unbounded => None,
        };
        let start = f(range.start);
        let end = f(range.end);

        Self { start, end }
    }
}

impl Validator for Reservation {
    fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.clone()));
        }

        validate_range(
            Some(self.start.as_ref().unwrap()),
            Some(self.end.as_ref().unwrap()),
        )?;
        Ok(())
    }
}
