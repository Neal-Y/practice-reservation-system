# Project Error Notes

- Feature Name: error notes
- Start Date: 2023-08-03 21:34:42

## Summary

Documenting the issues I faced and the strategies I employed to resolve them.

## Motivation

To chronicle the intriguing challenges I encountered during this project, detailing how I analyzed, dissected, and ultimately resolved them.

範本：
```
## ERROR：
### Describe:

## Error Analysis

## Solution
```

## ERROR： '?' couldn't convert problem
### Describe:
當我嘗試使用 '?' 返回錯誤時，我獲得了以下的錯誤訊息：

```rust
error[E0277]: `?` couldn't convert the error to `ReservationError`
  --> reservation/src/manager.rs:30:15
   |
30 |         .await?.get(0);
   |               ^ the trait `From<sqlx::Error>` is not implemented for `ReservationError`
   |
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
   = help: the following other types implement trait `FromResidual<R>`:
             <Result<T, F> as FromResidual<Result<Infallible, E>>>
             <Result<T, F> as FromResidual<Yeet<E>>>
   = note: required for `Result<Reservation, ReservationError>` to implement `FromResidual<Result<Infallible, sqlx::Error>>`

For more information about this error, try `rustc --explain E0277`.
```

## Error Analysis

從錯誤信息中，可以看到問題出在 'From' trait 沒有為 'ReservationError' 實現。Rust 中的 '?' 運算符會隱含地使用 'From' trait 對錯誤值進行轉換。

## Solution
可以進行普通的impl From trait，但由於想要更加靈活一點，我使用'thiserror' crate解決這問題
```rust
#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("database error")]
    DbError(#[from] sqlx::Error),
}
```

thiserror crate 提供了一個方便的方式來自定義你的錯誤類型，並自動實現 std::error::Error trait。#[from] 屬性告訴 thiserror 自動為 ReservationError::DbError 生成從 sqlx::Error 轉換來的代碼，這樣就可以使用 ? 運算符了。

簡單來說，sqlx::Error 自動轉換為你自定義的 ReservationError::DbError

## ERROR：message: "cannot cast type integer to rsvp.reservation_status"
### Describe:
```rust
thread 'manager::tests::reserve_should_work_for_valid_window' panicked at 'called `Result::unwrap()` on an `Err` value: DbError(Database(PgDatabaseError { severity: Error, code: "42846", message: "cannot cast type integer to rsvp.reservation_status", detail: None, hint: None, position: Some(Original(104)), where: None, schema: None, table: None, column: None, data_type: None, constraint: None, file: Some("parse_expr.c"), line: Some(2665), routine: Some("transformTypeCast") }))', reservation/src/manager.rs:95:48
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread 'manager::tests::reserve_should_work_for_valid_window' panicked at 'The main test function crashed, the test database got cleaned', reservation/src/manager.rs:80:5
```

## Error Analysis

可以看到是因為

```rust
#[derive(
    sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration,
)]
#[repr(i32)]
```

在這裡使用了 #[repr(i32)]，但是在資料庫中，reservation_status 是一個字串，所以出現這樣的錯誤。

## Solution

為ReservationStatus實現fmt::Display，並在資料庫中使用字串來表示reservation_status
