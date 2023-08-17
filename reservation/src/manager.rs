use abi::{Error, ReservationId, Validator};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, PgPool, Row};

use crate::{ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, Error> {
        rsvp.validate()?;

        let status = abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timestamp();

        let id:i64 = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id"
        )
        .bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.note.clone())
        .bind(status.to_string())
        .fetch_one(&self.pool)
        .await?.get(0);

        rsvp.id = id;

        Ok(rsvp)
    }

    // change reservation status
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        // error: code: "42883", message: "operator does not exist: uuid = text"，所以轉Uuid進去查詢語句。
        // let id: Uuid = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;

        id.validate()?;

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
        // let id = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;

        id.validate()?;

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
        // let id = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;

        id.validate()?;

        sqlx::query("DELETE FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // get reservation
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        // let id = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;

        id.validate()?;

        let rsvp = sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(rsvp)
    }

    async fn query(&self, query: abi::ReservationQuery) -> Result<Vec<abi::Reservation>, Error> {
        let user_id = str_to_option(&query.user_id);
        let resource_id = str_to_option(&query.resource_id);
        let range = query.get_timespan();
        let status = abi::ReservationStatus::from_i32(query.status)
            .unwrap_or(abi::ReservationStatus::Pending);
        let rsvps = sqlx::query_as(
            "select * from rsvp.query($1, $2, $3, $4::rsvp.reservation_status, $5, $6, $7)",
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(range)
        .bind(status.to_string())
        .bind(query.page)
        .bind(query.desc)
        .bind(query.page_size)
        .fetch_all(&self.pool)
        .await?;
        Ok(rsvps)
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
} // 創建一個新的 ReservationManager 實例，並將傳入的 pool 綁定到這個實例上。

fn str_to_option(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(test)]
mod tests {

    use abi::{
        Reservation, ReservationConflict, ReservationConflictInfo, ReservationQueryBuilder,
        ReservationWindow,
    };
    use prost_types::Timestamp;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let (rsvp, _manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;
        assert!(rsvp.id != 0);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_conflict_should_reject() {
        let (_rsvp, manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;

        let rsvp2 = abi::Reservation::new_pending(
            "conflict_userId",
            "Presidential-Suite",
            "2022-12-26T15:00:00+0800".parse().unwrap(),
            "2022-12-30T12:00:00+0800".parse().unwrap(),
            "Test Conflict",
        );

        let err = manager.reserve(rsvp2).await.unwrap_err();

        let info = ReservationConflictInfo::Parsed(ReservationConflict {
            new: ReservationWindow {
                rid: "Presidential-Suite".to_string(),
                start: "2022-12-26T15:00:00+0800".parse().unwrap(),
                end: "2022-12-30T12:00:00+0800".parse().unwrap(),
            },

            old: ReservationWindow {
                rid: "Presidential-Suite".to_string(),
                start: "2022-12-25T15:00:00+0800".parse().unwrap(),
                end: "2023-1-25T12:00:00+0800".parse().unwrap(),
            },
        });

        assert_eq!(err, abi::Error::ConflictReservation(info));
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn change_pending_status_should_be_confirm() {
        let (rsvp, manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;

        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        assert_eq!(rsvp.status, abi::ReservationStatus::Confirmed as i32);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn status_confirmed_update_status_should_do_nothing() {
        let (rsvp, manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;

        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        // update status again
        let rsvp = manager.change_status(rsvp.id).await.unwrap_err();

        assert_eq!(rsvp, abi::Error::NotFound);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_note_should_work() {
        let (rsvp, manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;
        let rsvp = manager.update_note(
            rsvp.id,
            "I spent all of my money so plz gives me a wonderful feeling. I want to have a wonderful experience.".to_string(),
        ).await.unwrap();

        assert_eq!(rsvp.note, "I spent all of my money so plz gives me a wonderful feeling. I want to have a wonderful experience.");
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn cancel_pending_status_should_be_cancelled() {
        let (rsvp, manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;
        manager.delete(rsvp.id).await.unwrap();
        let canceled = manager.get(rsvp.id).await.unwrap_err();

        assert_eq!(
            canceled,
            abi::Error::NotFound,
            "Failed to delete reservation"
        );
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn test_getter_should_return_reservation() {
        let (rsvp, manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;

        let rsvp = manager.get(rsvp.id).await.unwrap();

        assert_eq!(rsvp.status, abi::ReservationStatus::Pending as i32);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn test_query_should_return_vec_of_reservation() {
        let (rsvp, manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;
        let query = ReservationQueryBuilder::default()
            .user_id("yangid")
            .status(abi::ReservationStatus::Pending as i32)
            .start("2021-11-01T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2023-12-31T12:00:00+0800".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();

        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0], rsvp);

        // if timespan is not in a range, query should return empty

        let query = ReservationQueryBuilder::default()
            .user_id("yangid")
            .status(abi::ReservationStatus::Pending as i32)
            .start("2020-11-01T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2020-12-31T12:00:00+0800".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();

        let rsvps = manager.query(query).await.unwrap();

        assert!(rsvps.is_empty());

        // if status not match, should return empty
        let query = ReservationQueryBuilder::default()
            .user_id("yangid")
            .status(abi::ReservationStatus::Confirmed as i32)
            .start("2021-11-01T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2023-12-31T12:00:00+0800".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();
        let response = manager.query(query).await.unwrap();

        assert!(response.is_empty());

        // change status should be queryable
        manager.change_status(rsvp.id).await.unwrap();
        let query = ReservationQueryBuilder::default()
            .user_id("yangid")
            .status(abi::ReservationStatus::Confirmed as i32)
            .start("2021-11-01T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2023-12-31T12:00:00+0800".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();
        let query = manager.query(query).await.unwrap();
        assert_eq!(query.len(), 1);
    }

    async fn make_reservation_with_yang_template(
        pool: PgPool,
    ) -> (Reservation, ReservationManager) {
        make_reservation(
            pool,
            "yangid",
            "Presidential-Suite",
            "2022-12-25T15:00:00+0800",
            "2023-1-25T12:00:00+0800",
            "I spent all of my money so plz gives me a wonderful feeling.",
        )
        .await
    }

    async fn make_reservation(
        pool: PgPool,
        uid: &str,
        rid: &str,
        start: &str,
        end: &str,
        note: &str,
    ) -> (Reservation, ReservationManager) {
        let manager = ReservationManager::new(pool.clone());
        let rsvp = abi::Reservation::new_pending(
            uid,
            rid,
            start.parse().unwrap(),
            end.parse().unwrap(),
            note,
        );

        (manager.reserve(rsvp).await.unwrap(), manager)
    }
}
