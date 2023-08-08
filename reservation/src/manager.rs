use abi::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid, PgPool, Row};

use crate::{ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, Error> {
        rsvp.validate()?;

        let status = abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timestamp().into();

        let id:Uuid = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id"
        )
        .bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.note.clone())
        .bind(status.to_string())
        .fetch_one(&self.pool)
        .await?.get(0);

        rsvp.id = id.to_string();

        Ok(rsvp)
    }
    // change reservation status
    async fn change_status(&self, _id: ReservationId) -> Result<abi::Reservation, Error> {
        todo!()
    }
    // update note
    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, Error> {
        todo!()
    }
    // delete reservation
    async fn delete(&self, _id: ReservationId) -> Result<(), Error> {
        todo!()
    }
    // get reservation
    async fn get(&self, _id: ReservationId) -> Result<abi::Reservation, Error> {
        todo!()
    }
    async fn query(
        &self,
        _query_id: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, Error> {
        todo!()
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
} // 創建一個新的 ReservationManager 實例，並將傳入的 pool 綁定到這個實例上。

#[cfg(test)]
mod tests {

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());

        let rsvp = abi::Reservation::new_pending(
            "tryid",
            "ocean-view-room-713",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm. Please help to upgrade to executive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert!(!rsvp.id.is_empty());
    }

    // #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    // async fn reserve_conflict_should_reject() {
    //     let manager = ReservationManager::new(migrated_pool.clone());

    //     let rsvp1 = abi::Reservation::new_pending(
    //         "tryid",
    //         "ocean-view-room-713",
    //         "2022-12-25T15:00:00-0700".parse().unwrap(),
    //         "2022-12-28T12:00:00-0700".parse().unwrap(),
    //         "I'll arrive at 3pm. Please help to upgrade to executive room if possible.",
    //     );

    //     let rsvp2 = abi::Reservation::new_pending(
    //         "conflict_userId",
    //         "ocean-view-room-713",
    //         "2022-12-26T15:00:00-0700".parse().unwrap(),
    //         "2022-12-30T12:00:00-0700".parse().unwrap(),
    //         "Test Conflict",
    //     );

    //     let _rsvp1 = manager.reserve(rsvp1).await.unwrap();
    //     let err = manager.reserve(rsvp2).await.unwrap_err();
    //     println!("{:?}", err);
    //     if let abi::Error::ConflictReservation(_info) = err {}
    // }
}
