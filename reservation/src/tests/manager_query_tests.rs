#[cfg(test)]
mod tests {
    use crate::{tests::test_utils::*, Rsvp};
    use abi::ReservationQueryBuilder;
    use prost_types::Timestamp;

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

        let mut rx = manager.query(query).await;
        assert_eq!(rx.recv().await, Some(Ok(rsvp.clone())));
        assert_eq!(rx.recv().await, None);

        // if timespan is not in a range, query should return empty

        let query = ReservationQueryBuilder::default()
            .user_id("yangid")
            .status(abi::ReservationStatus::Pending as i32)
            .start("2020-11-01T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2020-12-31T12:00:00+0800".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();

        let mut rx = manager.query(query).await;
        assert_eq!(rx.recv().await, None);

        // if status not match, should return empty
        let query = ReservationQueryBuilder::default()
            .user_id("yangid")
            .status(abi::ReservationStatus::Confirmed as i32)
            .start("2021-11-01T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2023-12-31T12:00:00+0800".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();

        let mut rx = manager.query(query).await;
        assert_eq!(rx.recv().await, None);

        // change status should be queryable
        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        let query = ReservationQueryBuilder::default()
            .user_id("yangid")
            .status(abi::ReservationStatus::Confirmed as i32)
            .start("2021-11-01T15:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2023-12-31T12:00:00+0800".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();

        let mut rx = manager.query(query).await;
        assert_eq!(rx.recv().await, Some(Ok(rsvp)));
    }
}
