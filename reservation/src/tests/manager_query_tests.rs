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
}
