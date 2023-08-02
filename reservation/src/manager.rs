use async_trait::async_trait;
use chrono::{Date, DateTime, NaiveDate, Utc};
use sqlx::{postgres::types::PgRange, Row};

use crate::{ReservationError, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let start = abi::convert_time_to_utc(rsvp.start.unwrap());
        let end = abi::convert_time_to_utc(rsvp.end.unwrap());
        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let id = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id"
        )
        .bind(rsvp.user_id)
        .bind(rsvp.resource_id)
        .bind(timespan)
        .bind(rsvp.note)
        .bind(status)
        .fetch_one(&self.pool)
        .await?
        .get(0);
        Ok(())
    }
    // change reservation status
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }
    // update note
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }
    // delete reservation
    async fn delete(&self, id: ReservationId) -> Result<(), ReservationError> {
        todo!()
    }
    // get reservation
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }
}
