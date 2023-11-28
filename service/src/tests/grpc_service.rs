#[cfg(test)]
mod test {
    use crate::{start_the_server, tests::test_utils::TestConfig};
    use abi::{
        reservation_service_client::ReservationServiceClient, ConfirmRequest, FilterByIdBuilder,
        FilterRequest, FilterResponse, QueryRequest, Reservation, ReservationQueryBuilder,
        ReservationStatus, ReserveRequest,
    };
    use std::time::Duration;
    use tokio::time;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn internet_grpc_server_should_work() {
        let (config, mut client) =
            create_mock_database_and_start_server_with_diff_port(50000).await;

        // rsvp data
        let mut rsvp = Reservation::new_pending(
            "yangid",
            "Presidential-Suite",
            "2022-12-25T15:00:00+0800".parse().unwrap(),
            "2023-1-25T12:00:00+0800".parse().unwrap(),
            "integration test",
        );

        // test response from server is correct
        let response = client
            .reserve(ReserveRequest::new(rsvp.clone()))
            .await
            .unwrap()
            .into_inner()
            .reservation
            .unwrap();

        rsvp.id = response.id;
        assert_eq!(response, rsvp);

        // then test reservation is conflict
        let response = client.reserve(ReserveRequest::new(rsvp.clone())).await;

        assert!(response.is_err());

        // confirm first reservation
        let response = client
            .confirm(ConfirmRequest::new(rsvp.id))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(
            response.reservation.unwrap().status,
            ReservationStatus::Confirmed as i32
        );
        config.cleanup().await;
    }

    #[tokio::test]
    async fn grpc_filter_should_work() {
        let (config, mut client) =
            create_mock_database_and_start_server_with_diff_port(50001).await;

        make_reservations(&mut client, 100).await;

        // filter by user
        let filter = FilterByIdBuilder::default()
            .user_id("yang")
            .status(abi::ReservationStatus::Pending as i32)
            .build()
            .unwrap();

        let FilterResponse {
            pager,
            reservations,
        } = client
            .filter(FilterRequest::new(filter.clone()))
            .await
            .unwrap()
            .into_inner();

        let pager = pager.unwrap();

        // we already had an item, cuz rsvp_loop had reserved 100 items that means we have 10 page and other reserve new.
        assert_eq!(pager.next, filter.page_size);
        assert_eq!(pager.prev, -1);
        // assert_eq!(pager.total, 100); //TODO: not implemented yet
        assert_eq!(reservations.len(), filter.page_size as usize);

        let mut next_filter = filter.clone();
        next_filter.cursor = pager.clone().next;
        // then try get next page
        // let FilterResponse {
        //     pager,
        //     reservations,
        // } = client
        //     .filter(FilterRequest::new(next_filter.clone()))
        //     .await
        //     .unwrap()
        //     .into_inner();

        // assert_eq!(
        //     pager.clone().unwrap().next,
        //     next_filter.clone().cursor + filter.page_size
        // );
        // assert_eq!(pager.unwrap().prev, next_filter.cursor - 1);
        // assert_eq!(reservations.len(), filter.clone().page_size as usize);

        config.cleanup().await;
    }

    #[tokio::test]
    async fn grpc_query_should_work() {
        let (config, mut client) =
            create_mock_database_and_start_server_with_diff_port(50002).await;

        make_reservations(&mut client, 100).await;

        let query = ReservationQueryBuilder::default()
            .user_id("alice")
            .build()
            .unwrap();
        // query for all reservations
        let mut ret = client
            .query(QueryRequest::new(query))
            .await
            .unwrap()
            .into_inner();

        while let Some(Ok(rsvp)) = ret.next().await {
            assert_eq!(rsvp.user_id, "alice");
        }

        config.cleanup().await;
    }

    //? 同伺服器，不同port

    async fn create_mock_database_and_start_server_with_diff_port(
        port: u16,
    ) -> (
        TestConfig,
        ReservationServiceClient<tonic::transport::Channel>,
    ) {
        // pre-work
        let mut config = TestConfig::new().await;
        config.initialize().await;
        config.set_diff_port(port);

        let con = config.config.clone();

        // start the server
        tokio::spawn(async move {
            start_the_server(&con).await.unwrap();
        });
        time::sleep(Duration::from_millis(100)).await;

        // let client to make connections with server
        let client = ReservationServiceClient::connect(config.config.server.url(false))
            .await
            .unwrap();

        (config, client)
    }

    async fn make_reservations(
        client: &mut ReservationServiceClient<tonic::transport::Channel>,
        times: u32,
    ) {
        // then we make 100 reservation without conflict
        for i in 0..times {
            let mut rsvp_loop = Reservation::new_pending(
                "yang",
                format!("president-room-{}", i),
                "2022-12-25T15:00:00+0800".parse().unwrap(),
                "2023-1-25T12:00:00+0800".parse().unwrap(),
                &format!("integration test {}", i),
            );

            let response = client
                .reserve(ReserveRequest::new(rsvp_loop.clone()))
                .await
                .unwrap()
                .into_inner()
                .reservation
                .unwrap();

            rsvp_loop.id = response.id;

            assert_eq!(response, rsvp_loop);
        }
    }
}
