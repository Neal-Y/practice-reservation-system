use crate::ReservationStatus;
use std::fmt;

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Blocked => write!(f, "blocked"),
            Self::Confirmed => write!(f, "confirmed"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

// database equivalent of the "reservation_status" enum, translate RsvpStatus into database's reservation_status.
// cuz database's reservation_status have #[repr(i32)] represent i32 in FFI(外部函數介面)
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Pending,
    Blocked,
    Confirmed,
    Unknown,
}

impl From<RsvpStatus> for ReservationStatus {
    fn from(status: RsvpStatus) -> Self {
        match status {
            RsvpStatus::Pending => Self::Pending,
            RsvpStatus::Blocked => Self::Blocked,
            RsvpStatus::Confirmed => Self::Confirmed,
            RsvpStatus::Unknown => Self::Unknown,
        }
    }
}
