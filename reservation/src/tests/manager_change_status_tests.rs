#[cfg(test)]
mod tests {
    use crate::{tests::test_utils::*, Rsvp};

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
}
