#[cfg(test)]
mod test {
    use crate::start_the_server;
    use abi::{
        reservation_service_client::ReservationServiceClient, Config, ConfirmRequest,
        FilterByIdBuilder, FilterRequest, FilterResponse, Reservation, ReservationStatus,
        ReserveRequest,
    };
    use std::time::Duration;
    use tokio::time;

    #[tokio::test]
    async fn grpc_server_should_work() {
        // pre-work
        let config = Config::load("./fixtures/config.yml").unwrap();
        let config_clone = config.clone();

        // rsvp data
        let rsvp = Reservation::new_pending(
            "yangid",
            "Presidential-Suite",
            "2022-12-25T15:00:00+0800".parse().unwrap(),
            "2023-1-25T12:00:00+0800".parse().unwrap(),
            "integration test",
        );

        // start the server
        tokio::spawn(async move {
            start_the_server(&config_clone).await.unwrap();
        });
        time::sleep(Duration::from_millis(100)).await;

        // let client to make connections with server
        let mut client = ReservationServiceClient::connect(config.server.url(false))
            .await
            .unwrap();

        // test response from server is correct
        let response = client
            .reserve(ReserveRequest::new(rsvp.clone()))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.reservation, Some(rsvp.clone()));

        // then test reservation is conflict
        let response = client.reserve(ReserveRequest::new(rsvp.clone())).await;

        assert_eq!(response.unwrap_err().to_string(), "error conflict.");

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

        // then we make 100 reservation without conflict
        for i in 0..100 {
            let rsvp_loop = Reservation::new_pending(
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
                .into_inner();
            assert_eq!(response.reservation, Some(rsvp_loop));
        }

        // filter by user
        let filter = FilterByIdBuilder::default()
            .user_id("yang")
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

        assert_eq!(pager.clone().unwrap().next, filter.clone().page_size);
        assert_eq!(pager.unwrap().prev, -1);
        assert_eq!(reservations.len(), filter.clone().page_size as usize);
    }
}
