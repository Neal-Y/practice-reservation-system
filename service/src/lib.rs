mod service;
mod tests;

use abi::{reservation_service_server::ReservationServiceServer, Config, Reservation};
use anyhow::Ok;
use futures::Stream;
use reservation::ReservationManager;
use std::pin::Pin;
use tonic::{transport::Server, Status};

pub struct RsvpService {
    manager: ReservationManager,
}

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

pub async fn start_the_server(config: &Config) -> Result<(), anyhow::Error> {
    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let service = RsvpService::from_config(config).await?;
    let service = ReservationServiceServer::new(service);

    Server::builder().add_service(service).serve(addr).await?;
    Ok(())
}
