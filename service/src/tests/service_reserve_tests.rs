#[cfg(test)]
mod test {
    use crate::{tests::test_utils::TestConfig, RsvpService};
    use abi::{reservation_service_server::ReservationService, Reservation, ReserveRequest};

    #[tokio::test]
    async fn local_test_rpc_reserve_server_should_work() {
        let config = TestConfig::new().await;
        config.initialize().await;

        let service = RsvpService::from_config(&config.config).await.unwrap();
        let reservation = Reservation::new_pending(
            "yangid",
            "Presidential-Suite",
            "2022-12-25T15:00:00+0800".parse().unwrap(),
            "2023-1-25T12:00:00+0800".parse().unwrap(),
            "test this MDFK",
        );

        let request = tonic::Request::new(ReserveRequest {
            reservation: Some(reservation.clone()),
        });

        let response = service.reserve(request).await.unwrap();

        let receive_from_response_reservation = response.into_inner().reservation;

        assert!(receive_from_response_reservation.is_some()); // 一定是他媽有點東西的！ absolutely not None MDFK.

        let receive_from_response = receive_from_response_reservation.unwrap();

        assert_eq!(receive_from_response.user_id, reservation.user_id);
        assert_eq!(receive_from_response.resource_id, reservation.resource_id);
        assert_eq!(receive_from_response.start, reservation.start);
        assert_eq!(receive_from_response.end, reservation.end);
        assert_eq!(receive_from_response.note, reservation.note);

        config.cleanup().await;
    }
}
