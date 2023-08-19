#[cfg(test)]
mod tests {
    use crate::{tests::test_utils::*, Rsvp};

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
}
