mod error;
mod pb;
mod types;
mod utils;

pub use error::{Error, ReservationConflict, ReservationConflictInfo, ReservationWindow};
pub use pb::*;
pub use types::*;

pub trait Validator {
    fn validate(&self) -> Result<(), Error>;
}
