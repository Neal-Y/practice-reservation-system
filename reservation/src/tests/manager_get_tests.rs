#[cfg(test)]
mod tests {
    use crate::{tests::test_utils::*, Rsvp};

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn test_getter_should_return_reservation() {
        let (rsvp, manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;

        let rsvp = manager.get(rsvp.id).await.unwrap();

        assert_eq!(rsvp.status, abi::ReservationStatus::Pending as i32);
    }
}
