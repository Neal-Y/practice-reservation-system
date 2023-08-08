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

簡單來說，sqlx::Error 自動轉換為你自定義的ReservationError::DbError

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

## ERROR：during Pre-commit git has tracked compiled files lead to cargo hook conflict
### Describe:
當我使用*pre-commit hook*來讓我在*git commit -a*時可以自動運行測試，但是在我使用*git commit -a*時，出現諸多conflict，舉凡像是：
1. end-of-file-fixer： hook 誤以為target檔有錯誤所以自行幫我加上一些修復像是
````Fixing target/debug/.fingerprint/try-lock-bb4d5568da5b854e/lib-try-lock.json````
2. Typos 語法錯誤： 同上像是：
```rust
typos....................................................................Failed
- hook id: typos
- exit code: 2

error: `ba` should be `by`, `be`
  --> target/debug/.fingerprint/futures-io-0fdc36943930b64b/lib-futures-io:1:11
  |
1 | dfce884c71ba0318
  |           ^^
  |
error: `ba` should be `by`, `be`
````
3. cargo clippy、cargo check、cargo test conflict：如上所說，導致他們對於已編譯的檔案不相符需要重編譯，而這就導致
``` files were modified by this hook```

## Error Analysis

首先，cargo clippy、cargo check、cargo test他們各自由於需要深入了解代碼的語法和語義，所以它實際上會執行類似於```編譯的過程```。但是，這與 cargo build 不同，他們不會產生最終的二進制輸出。

cargo clippy 和 cargo check 都可能與 target 目錄中的編譯緩存互動，這就是為什麼會看到```"file were modified by this hook"```的原因。

也因為這樣導致當我有追蹤原本就編譯好的target/檔會出現不符預期各個hook需要重新編譯的原因。

## Solution

當執行了```git rm -r --cached target```命令，其實是告訴 Git 忽略 target 目錄下的所有已經追蹤的文件。這樣做的結果是，這些文件不再被 Git 控制，所以它們不會被包括在任何提交或檢查中。

以下是這樣做可能解決了問題的原因：

1. **重複編譯問題**: 由於 target 目錄不再被 Git 控制，每個 pre-commit hook 都可以自由修改它，而不會讓 Git 察覺這些變化。這樣就不會觸發 "files were modified by this hook" 的錯誤。

2. **編譯產物和源代碼的分離**: target 目錄通常包括編譯的產物，而不是源代碼。一些工具（例如 cargo clippy 或 cargo check）可能會對 target 目錄中的文件進行不必要的操作，因為它們誤以為這些文件是源代碼的一部分。通過讓 Git 忽略這個目錄，也告訴這些工具忽略它，從而避免了這個問題。

3. **typos 出錯**: typos 是一個拼寫檢查工具。如果它被配置為檢查 target 目錄，它可能會在編譯的產物中找到 "拼寫錯誤"，這實際上可能只是源代碼中合法的符號或識別符。通過讓 Git 忽略 target 目錄，也避免了這個問題。

最後我在.gitignore 文件中添加/target，以便 Git 自動忽略這個目錄。這樣，不僅可以避免上述問題，而且還可以確保未來的 ``git add . `` 命令不會意外地將這個目錄加入到存儲庫中。
