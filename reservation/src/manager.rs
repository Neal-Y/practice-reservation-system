use abi::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid, PgPool, Row};
use std::str::FromStr;

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
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        // error: code: "42883", message: "operator does not exist: uuid = text"，所以轉Uuid進去查詢語句。
        let id = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;

        // if current status is pending, change status into confirmed, otherwise do nothing
        let rsvp = sqlx::query_as("UPDATE rsvp.reservations SET status = 'confirmed' WHERE id = $1 AND status = 'pending' RETURNING *
        ")
            .bind(id)
            .fetch_one(&self.pool)
            .await.map_err(|_|abi::Error::NotFound)?;

        Ok(rsvp)
    }

    // update note
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, Error> {
        let id = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp =
            sqlx::query_as("UPDATE rsvp.reservations SET note = $1 WHERE id = $2 RETURNING *")
                .bind(note)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(rsvp)
    }

    // delete reservation
    async fn delete(&self, id: ReservationId) -> Result<(), Error> {
        let id = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        sqlx::query("DELETE FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // get reservation
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        let id = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp = sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(rsvp)
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

    use abi::{ReservationConflict, ReservationConflictInfo, ReservationWindow};

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

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_conflict_should_reject() {
        let manager = ReservationManager::new(migrated_pool.clone());

        let rsvp1 = abi::Reservation::new_pending(
            "tryid",
            "ocean-view-room-713",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm. Please help to upgrade to executive room if possible.",
        );

        let rsvp2 = abi::Reservation::new_pending(
            "conflict_userId",
            "ocean-view-room-713",
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "Test Conflict",
        );

        let _rsvp1 = manager.reserve(rsvp1).await.unwrap();
        let err = manager.reserve(rsvp2).await.unwrap_err();

        let info = ReservationConflictInfo::Parsed(ReservationConflict {
            a: ReservationWindow {
                rid: "ocean-view-room-713".to_string(),
                start: "2022-12-26T15:00:00-0700".parse().unwrap(),
                end: "2022-12-30T12:00:00-0700".parse().unwrap(),
            },
            b: ReservationWindow {
                rid: "ocean-view-room-713".to_string(),
                start: "2022-12-25T15:00:00-0700".parse().unwrap(),
                end: "2022-12-28T12:00:00-0700".parse().unwrap(),
            },
        });

        assert_eq!(err, abi::Error::ConflictReservation(info));
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn change_pending_status_should_be_confirm() {
        let manager = ReservationManager::new(migrated_pool.clone());

        let rsvp = abi::Reservation::new_pending(
            "yangid",
            "Presidential Suite",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2023-1-25T12:00:00-0700".parse().unwrap(),
            "I spent all of my money so plz gives me a wonderful feeling.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        assert_eq!(rsvp.status, abi::ReservationStatus::Confirmed as i32);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn status_confirmed_update_status_should_do_nothing() {
        let manager = ReservationManager::new(migrated_pool.clone());

        let rsvp = abi::Reservation::new_pending(
            "yangid",
            "Presidential Suite",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2023-1-25T12:00:00-0700".parse().unwrap(),
            "I spent all of my money so plz gives me a wonderful feeling.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        // update status again
        let rsvp = manager.change_status(rsvp.id).await.unwrap_err();

        assert_eq!(rsvp, abi::Error::NotFound);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_note_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());

        let rsvp = abi::Reservation::new_pending(
            "yangid",
            "Presidential Suite",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2023-1-25T12:00:00-0700".parse().unwrap(),
            "I spent all of my money so plz gives me a wonderful feeling.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        let rsvp = manager.update_note(
            rsvp.id,
            "I spent all of my money so plz gives me a wonderful feeling. I want to have a wonderful experience.".to_string(),
        ).await.unwrap();

        assert_eq!(rsvp.note, "I spent all of my money so plz gives me a wonderful feeling. I want to have a wonderful experience.");
    }

    // #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    // async fn cancel_pending_status_should_be_cancelled() {
    //     let manager = ReservationManager::new(migrated_pool.clone());

    //     let rsvp = abi::Reservation::new_pending(
    //         "yangid",
    //         "Presidential Suite",
    //         "2022-12-25T15:00:00-0700".parse().unwrap(),
    //         "2023-1-25T12:00:00-0700".parse().unwrap(),
    //         "I spent all of my money so plz gives me a wonderful feeling.",
    //     );

    //     let rsvp = manager.reserve(rsvp).await.unwrap();

    //     let rsvp = manager.delete(rsvp.id).await.unwrap();

    //     assert_eq!(rsvp, ());
    // }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn test_getter_should_return_reservation() {
        let manager = ReservationManager::new(migrated_pool.clone());

        let rsvp = abi::Reservation::new_pending(
            "yangid",
            "Presidential Suite",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2023-1-25T12:00:00-0700".parse().unwrap(),
            "I spent all of my money so plz gives me a wonderful feeling.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        let rsvp = manager.get(rsvp.id).await.unwrap();

        assert_eq!(rsvp.status, abi::ReservationStatus::Pending as i32);
    }
}

// async fn make_reservation(
//     pool: PgPool,
//     id: &str,
//     rid: &str,
//     start: &str,
//     end: &str,
//     note: &str,
// ) -> Reservation {
//     let manager = Arc::new(ReservationManager::new(pool.clone()));
//     let rsvp =
//         abi::Reservation::new_pending(id, rid, start.parse().unwrap(), end.parse().unwrap(), note)?;
//     manager.reserve(rsvp).await?
// }
