//? 在這層我只需要將原本寫好的reservation的核心功能將他們與grpc的api串接起來，做簡單的IO處理，這樣就是對外的gRPC interface了
//? 把吃進來的protobuf定義的資料轉成reservation core的資料再將他們輸出出去即可
//? 輸入->校驗->轉換(required args by reservation core)->處理->轉換(gRPC interface)->輸出

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{ReservationStream, RsvpService, TonicReceiverStream};
use abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, Config,
    ConfirmRequest, ConfirmResponse, FilterRequest, FilterResponse, GetRequest, GetResponse,
    ListenRequest, QueryRequest, ReserveRequest, ReserveResponse, UpdateRequest, UpdateResponse,
};
use futures::Stream;
use reservation::{ReservationManager, Rsvp};
use tokio::sync::mpsc;
use tonic::{async_trait, Request, Response, Status};

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
        request: Request<ConfirmRequest>,
    ) -> std::result::Result<Response<ConfirmResponse>, Status> {
        let request = request.into_inner();
        let confirm = self.manager.change_status(request.id).await?;
        Ok(Response::new(ConfirmResponse {
            reservation: Some(confirm),
        }))
    }

    /// update the reservation note
    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> std::result::Result<Response<UpdateResponse>, Status> {
        let request = request.into_inner();
        let update = self.manager.update_note(request.id, request.note).await?;
        Ok(Response::new(UpdateResponse {
            reservation: Some(update),
        }))
    }
    /// cancel a reservation
    async fn cancel(
        &self,
        request: Request<CancelRequest>,
    ) -> std::result::Result<Response<CancelResponse>, Status> {
        let request = request.into_inner();
        let delete = self.manager.delete(request.id).await?;
        Ok(Response::new(CancelResponse {
            reservation: Some(delete),
        }))
    }
    /// get a reservation by id
    async fn get(
        &self,
        request: Request<GetRequest>,
    ) -> std::result::Result<Response<GetResponse>, Status> {
        let request = request.into_inner();
        let get = self.manager.get(request.id).await?;
        Ok(Response::new(GetResponse {
            reservation: Some(get),
        }))
    }

    /// Server streaming response type for the query method.
    type queryStream = ReservationStream;
    /// query reservations by resource id, user id, status, start time, end time
    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> std::result::Result<Response<Self::queryStream>, Status> {
        let request = request.into_inner();
        if request.query.is_none() {
            return Err(Status::invalid_argument("query is failed"));
        }
        let query = self.manager.query(request.query.unwrap()).await;
        let stream = TonicReceiverStream::new(query);

        Ok(Response::new(Box::pin(stream)))
    }

    /// filter reservations order by reservation id
    async fn filter(
        &self,
        request: Request<FilterRequest>,
    ) -> std::result::Result<Response<FilterResponse>, Status> {
        let request = request.into_inner();
        if request.query.is_none() {
            return Err(Status::invalid_argument("query is required"));
        }
        let filter = self.manager.keyset_query(request.query.unwrap()).await?;
        Ok(Response::new(FilterResponse {
            reservations: filter.1,
            pager: Some(filter.0),
        }))
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

// in order to turn mpsc::Receiver<Result<T, abi::Error>> into tonic::Response
impl<T> Stream for TonicReceiverStream<T> {
    type Item = Result<T, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_recv(cx) {
            Poll::Ready(Some(Ok(t))) => Poll::Ready(Some(Ok(t))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))), // cuz implement the From trait for abi::Error
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

// just new function
impl<T> TonicReceiverStream<T> {
    pub fn new(inner: mpsc::Receiver<Result<T, abi::Error>>) -> Self {
        Self { inner }
    }
}
