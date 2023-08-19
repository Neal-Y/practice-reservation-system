#[cfg(test)]
mod tests {
    use crate::{tests::test_utils::*, Rsvp};
    use abi::FilterByIdBuilder;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn test_filter_query_should_return_vec_of_reservation() {
        let (rsvp, manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;

        let filter = FilterByIdBuilder::default()
            .user_id("yangid")
            .status(abi::ReservationStatus::Pending as i32)
            .build()
            .unwrap();

        let (pager, rsvps) = manager.keyset_query(filter).await.unwrap();

        // 只有一筆資料，所以prev跟next都是-1
        assert_eq!(pager.prev, -1);
        assert_eq!(pager.next, -1);
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0], rsvp);
    }
}
