use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{
    convert_timestamp_into_timespan_pgrange, utils::convert_to_timestamp, validate_range, Error,
    ReservationQuery, ReservationStatus, Validator,
};

#[allow(clippy::too_many_arguments)]
impl ReservationQuery {
    pub fn new(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        status: ReservationStatus,
        page: i32,
        is_desc: bool,
        page_size: i32,
    ) -> Self {
        Self {
            user_id: uid.into(),
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(start)),
            end: Some(convert_to_timestamp(end)),
            status: status as i32,
            page,
            page_size,
            desc: is_desc,
        }
    }

    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        convert_timestamp_into_timespan_pgrange(
            Some(self.start.as_ref().unwrap()),
            Some(self.end.as_ref().unwrap()),
        )
    }
}

impl Validator for ReservationQuery {
    fn validate(&self) -> Result<(), Error> {
        validate_range(
            Some(self.start.as_ref().unwrap()),
            Some(self.end.as_ref().unwrap()),
        )?;

        Ok(())
    }
}
