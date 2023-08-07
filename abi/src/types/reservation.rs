use chrono::{DateTime, FixedOffset, Utc};
use std::ops::Range;

use crate::{
    utils::{convert_time_to_utc, convert_to_timestamp},
    Error, Reservation, ReservationStatus,
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
            id: "".to_string(),
            user_id: uid.into(),
            status: ReservationStatus::Pending as i32,
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: note.into(),
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.start.is_none() || self.end.is_none() {
            return Err(Error::InvalidTime);
        }

        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.clone()));
        }
        let start = convert_time_to_utc(self.start.as_ref().unwrap().clone());
        let end = convert_time_to_utc(self.end.as_ref().unwrap().clone());

        if start >= end {
            return Err(Error::InvalidTime);
        }

        Ok(())
    }

    pub fn get_timestamp(&self) -> Range<DateTime<Utc>> {
        let start = convert_time_to_utc(self.start.as_ref().unwrap().clone());
        let end = convert_time_to_utc(self.end.as_ref().unwrap().clone());
        Range { start, end }
    }
}
