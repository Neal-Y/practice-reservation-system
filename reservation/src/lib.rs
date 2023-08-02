mod error;
mod manager;

use async_trait::async_trait;
pub use error::ReservationError;
use sqlx::PgPool;

pub type ReservationId = String;
pub type ResourceId = String;

pub struct ReservationManager {
    pool: PgPool, // sqlx 裡面 postgres pool database connection 使用Arc將各種database connection 分開
}

#[async_trait]
pub(crate) trait Rsvp {
    // make a reservation
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, ReservationError>;
    // change reservation status
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError>;
    // update note
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, ReservationError>;
    // delete reservation
    async fn delete(&self, id: ReservationId) -> Result<(), ReservationError>;
    // get reservation
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError>;
    // get user's all reservation
    async fn query(
        &self,
        query_id: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError>;
}
