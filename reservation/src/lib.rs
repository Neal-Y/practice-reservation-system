mod manager;
mod tests;

use abi::{Error, FilterPager};
use async_trait::async_trait;
pub use manager::ReservationManager;

#[async_trait]
pub(crate) trait Rsvp {
    // make a reservation
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, Error>;
    // change reservation status
    async fn change_status(&self, id: abi::ReservationId) -> Result<abi::Reservation, Error>;
    // update note
    async fn update_note(
        &self,
        id: abi::ReservationId,
        note: String,
    ) -> Result<abi::Reservation, Error>;
    // delete reservation
    async fn delete(&self, id: abi::ReservationId) -> Result<(), Error>;
    // get reservation
    async fn get(&self, id: abi::ReservationId) -> Result<abi::Reservation, Error>;
    // get user's all reservation
    async fn query(&self, query_id: abi::ReservationQuery) -> Result<Vec<abi::Reservation>, Error>;
    // query reservation order by reservation id
    async fn keyset_query(
        &self,
        filter: abi::FilterById,
    ) -> Result<(FilterPager, Vec<abi::Reservation>), Error>;
}
