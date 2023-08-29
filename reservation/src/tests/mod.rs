// mod manager_change_status_tests;
// mod manager_delete_tests;
// mod manager_filter_tests;
// mod manager_get_tests;
// mod manager_query_tests;
// mod manager_reserve_tests;
// mod manager_update_note_tests;

#[cfg(test)]
mod test_utils {
    use crate::{ReservationManager, Rsvp};
    use abi::Reservation;
    use sqlx::PgPool;

    #[allow(dead_code)]
    pub async fn make_reservation_with_yang_template(
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

    pub async fn make_reservation(
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
