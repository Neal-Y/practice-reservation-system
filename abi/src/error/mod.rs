mod conflict;

use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

pub use conflict::{ReservationConflict, ReservationConflictInfo, ReservationWindow};

#[derive(Error, Debug)]
pub enum Error {
    #[error("pending was confirmed")]
    NotFound,

    #[error("failed to parse config file")]
    FailedToParse,

    #[error("failed to read file")]
    FailedToRead,

    #[error("database error")]
    DbError(sqlx::Error),

    #[error("Invalid start or end time for reservation")]
    InvalidTime,

    #[error("Invalid User Id:{0}")]
    InvalidUserId(String),

    #[error("Invalid Reservation Id:{0}")]
    InvalidReservationId(i64),

    #[error("conflict reservation")]
    ConflictReservation(ReservationConflictInfo),

    #[error("Invalid Resource Id:{0}")]
    InvalidResourceId(String),

    #[error("unknown error")]
    Unknown,

    #[error("parsed failed")]
    ParsedFailed,
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictReservation(err.detail().unwrap().parse().unwrap())
                    }
                    _ => Error::DbError(sqlx::Error::Database(e)),
                }
            }
            sqlx::Error::RowNotFound => Error::NotFound,
            _ => Error::DbError(e),
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DbError(_), Self::DbError(_)) => true,
            (Self::InvalidTime, Self::InvalidTime) => true,
            (Self::InvalidUserId(v1), Self::InvalidUserId(v2)) => v1 == v2,
            (Self::InvalidReservationId(v1), Self::InvalidReservationId(v2)) => v1 == v2,
            (Self::ConflictReservation(v1), Self::ConflictReservation(v2)) => v1 == v2,
            (Self::InvalidResourceId(v1), Self::InvalidResourceId(v2)) => v1 == v2,
            (Self::Unknown, Self::Unknown) => true,
            (Self::ParsedFailed, Self::ParsedFailed) => true,
            (Self::NotFound, Self::NotFound) => true,
            _ => false,
        }
    }
}

impl From<crate::Error> for tonic::Status {
    fn from(e: crate::Error) -> Self {
        match e {
            crate::Error::NotFound => tonic::Status::not_found("not found"),
            crate::Error::InvalidTime => tonic::Status::invalid_argument("invalid time"),
            crate::Error::InvalidUserId(v) => {
                tonic::Status::invalid_argument(format!("invalid user id: {}", v))
            }
            crate::Error::InvalidReservationId(v) => {
                tonic::Status::invalid_argument(format!("invalid reservation id: {}", v))
            }
            crate::Error::ConflictReservation(v) => {
                tonic::Status::failed_precondition(format!("conflict reservation: {:?}", v))
            }
            crate::Error::InvalidResourceId(v) => {
                tonic::Status::invalid_argument(format!("invalid resource id: {}", v))
            }
            crate::Error::ParsedFailed => tonic::Status::unknown("parsed failed"),
            crate::Error::FailedToParse => tonic::Status::unknown("failed to parse"),
            crate::Error::FailedToRead => tonic::Status::unknown("failed to read"),
            crate::Error::DbError(e) => tonic::Status::internal(format!("Database error: {}", e)),
            crate::Error::Unknown => tonic::Status::unknown("unknown"),
        }
    }
}
