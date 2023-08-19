#[cfg(test)]
mod tests {
    use crate::{tests::test_utils::*, Rsvp};
    use abi::{ReservationConflict, ReservationConflictInfo, ReservationWindow};

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
}
