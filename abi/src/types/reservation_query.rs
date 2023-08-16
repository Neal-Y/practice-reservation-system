use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{
    convert_timestamp_into_timespan_pgrange, validate_range, Error, ReservationQuery, Validator,
};

impl ReservationQuery {
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
