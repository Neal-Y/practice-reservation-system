use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader {
    //     expected: String,
    //     found: String,
    // },
    #[error("database error")]
    DbError(sqlx::Error),

    #[error("Invalid start or end time for reservation")]
    InvalidTime,

    #[error("Invalid User Id:{0}")]
    InvalidUserId(String),

    #[error("{0}")]
    ConflictReservation(String),

    #[error("Invalid Resource Id:{0}")]
    InvalidResourceId(String),

    #[error("unknown error")]
    Unknown,
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictReservation(err.detail().unwrap().to_string())
                    }
                    _ => Error::DbError(sqlx::Error::Database(e)),
                }
            }
            _ => Error::DbError(e),
        }
    }
}
// 我想返回一個很“清晰”衝突問題給用戶而不是這樣
/*
ConflictReservation("Key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\"))
conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\")).")
test manager::tests::reserve_conflict_should_reject ... ok
*/

// 那由於PgDatabaseError也沒有其他方法提供一個更好的錯誤信息，我們只能自己implement function從 PgDatabaseError中的 "get_raw()"取得一大坨原始資料並解析它。

// TODO: write a parser
