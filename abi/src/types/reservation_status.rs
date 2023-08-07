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
