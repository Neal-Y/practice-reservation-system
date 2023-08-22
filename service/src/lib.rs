//? 在這層我只需要將原本寫好的reservation的核心功能將他們與grpc的api串接起來，做簡單的IO處理，這樣就是對外的gRPC interface了
//? 把吃進來的protobuf定義的資料轉成reservation core的資料再將他們輸出出去即可
//? 輸入->校驗->轉換(required args by reservation core)->處理->轉換(gRPC interface)->輸出

use abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, ConfirmRequest,
    ConfirmResponse, FilterRequest, FilterResponse, GetRequest, GetResponse, ListenRequest,
    QueryRequest, ReserveRequest, ReserveResponse, UpdateRequest, UpdateResponse,
};
use abi::{Config, Reservation};
use futures::Stream;
use reservation::{ReservationManager, Rsvp};
use std::pin::Pin;
use tonic::{async_trait, Request, Response, Status};

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

pub struct RsvpService {
    manager: ReservationManager,
}

impl RsvpService {
    pub async fn from_config(config: &Config) -> Result<Self, anyhow::Error> {
        Ok(Self {
            manager: ReservationManager::from_config(&config.db).await?,
        })
    }
}

#[async_trait]
impl ReservationService for RsvpService {
    /// make a reservation
    async fn reserve(
        &self,
        request: Request<ReserveRequest>,
    ) -> std::result::Result<Response<ReserveResponse>, Status> {
        let request = request.into_inner();
        if request.reservation.is_none() {
            return Err(Status::invalid_argument("reservation is required"));
        }
        let reservation = self.manager.reserve(request.reservation.unwrap()).await?;
        Ok(Response::new(ReserveResponse {
            reservation: Some(reservation),
        }))
    }
    /// confirm a pending reservation, if reservation is not pending, do nothing
    async fn confirm(
        &self,
        _request: Request<ConfirmRequest>,
    ) -> std::result::Result<Response<ConfirmResponse>, Status> {
        todo!()
    }
    /// update the reservation note
    async fn update(
        &self,
        _request: Request<UpdateRequest>,
    ) -> std::result::Result<Response<UpdateResponse>, Status> {
        todo!()
    }
    /// cancel a reservation
    async fn cancel(
        &self,
        _request: Request<CancelRequest>,
    ) -> std::result::Result<Response<CancelResponse>, Status> {
        todo!()
    }
    /// get a reservation by id
    async fn get(
        &self,
        _request: Request<GetRequest>,
    ) -> std::result::Result<Response<GetResponse>, Status> {
        todo!()
    }
    /// Server streaming response type for the query method.
    type queryStream = ReservationStream;
    /// query reservations by resource id, user id, status, start time, end time
    async fn query(
        &self,
        _request: Request<QueryRequest>,
    ) -> std::result::Result<Response<Self::queryStream>, Status> {
        todo!()
    }
    /// filter reservations order by reservation id
    async fn filter(
        &self,
        _request: Request<FilterRequest>,
    ) -> std::result::Result<Response<FilterResponse>, Status> {
        todo!()
    }
    /// Server streaming response type for the listen method.
    type listenStream = ReservationStream;
    /// another system could monitor newly added/confirmed/cancelled reservations
    async fn listen(
        &self,
        _request: Request<ListenRequest>,
    ) -> std::result::Result<Response<Self::listenStream>, Status> {
        todo!()
    }
}
