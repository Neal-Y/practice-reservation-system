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
``Fixing target/debug/.fingerprint/try-lock-bb4d5568da5b854e/lib-try-lock.json``
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
```
3. cargo clippy、cargo check、cargo test conflict：如上所說，導致他們對於已編譯的檔案不相符需要重編譯，而這就導致
`` files were modified by this hook``

## Error Analysis

首先，cargo clippy、cargo check、cargo test他們各自由於需要深入了解代碼的語法和語義，所以它實際上會執行類似於```編譯的過程```。但是，這與 cargo build 不同，他們不會產生最終的二進制輸出。

cargo clippy 和 cargo check 都可能與 target 目錄中的編譯緩存互動，這就是為什麼會看到``"file were modified by this hook"``的原因。

也因為這樣導致當我有追蹤原本就編譯好的target/檔會出現不符預期各個hook需要重新編譯的原因。

## Solution

當執行了```git rm -r --cached target```命令，其實是告訴 Git 忽略 target 目錄下的所有已經追蹤的文件。這樣做的結果是，這些文件不再被 Git 控制，所以它們不會被包括在任何提交或檢查中。

以下是這樣做可能解決了問題的原因：

1. **重複編譯問題**: 由於 target 目錄不再被 Git 控制，每個 pre-commit hook 都可以自由修改它，而不會讓 Git 察覺這些變化。這樣就不會觸發 "files were modified by this hook" 的錯誤。

2. **編譯產物和源代碼的分離**: target 目錄通常包括編譯的產物，而不是源代碼。一些工具（例如 cargo clippy 或 cargo check）可能會對 target 目錄中的文件進行不必要的操作，因為它們誤以為這些文件是源代碼的一部分。通過讓 Git 忽略這個目錄，也告訴這些工具忽略它，從而避免了這個問題。

3. **typos 出錯**: typos 是一個拼寫檢查工具。如果它被配置為檢查 target 目錄，它可能會在編譯的產物中找到 "拼寫錯誤"，這實際上可能只是源代碼中合法的符號或識別符。通過讓 Git 忽略 target 目錄，也避免了這個問題。

最後我在.gitignore 文件中添加/target，以便 Git 自動忽略這個目錄。這樣，不僅可以避免上述問題，而且還可以確保未來的 ``git add . `` 命令不會意外地將這個目錄加入到存儲庫中。

## ERROR： trait `for<'r> FromRow<'r, PgRow>` is not implemented for `Reservation`
### Describe: 當我對change_status()進行implement時，中間針對資料庫的操作時我使用了sqlx::query_as()，但是在編譯時出現了以下錯誤：
```rust
error[E0277]: the trait bound `for<'r> Reservation: FromRow<'r, PgRow>` is not satisfied
   --> reservation/src/manager.rs:38:20
    |
38  |         let rsvp = sqlx::query_as("UPDATE rsvp.reservation.status SET status = 'confirmed' WHERE id = $1 AND status = 'pending' RETURNING *")
    |                    ^^^^^^^^^^^^^^ the trait `for<'r> FromRow<'r, PgRow>` is not implemented for `Reservation`
    |
    = help: the following other types implement trait `FromRow<'r, R>`:
              (T1, T2)
              (T1, T2, T3)
              (T1, T2, T3, T4)
              (T1, T2, T3, T4, T5)
              (T1, T2, T3, T4, T5, T6)
              (T1, T2, T3, T4, T5, T6, T7)
              (T1, T2, T3, T4, T5, T6, T7, T8)
              (T1, T2, T3, T4, T5, T6, T7, T8, T9)
            and 8 others
note: required by a bound in `sqlx::query_as`
   --> /Users/shin/.cargo/registry/src/github.com-1ecc6299db9ec823/sqlx-core-0.6.3/src/query_as.rs:174:8
    |
174 |     O: for<'r> FromRow<'r, DB::Row>,
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `query_as`

For more information about this error, try `rustc --explain E0277`.
```

## Error Analysis

簡單來說就是`Reservation這個 struct 沒有實作FromRow<'r, PgRow>`這個trait，但是我在實作時卻有使用到sqlx::query_as()，而這個方法需要實作FromRow<'r, PgRow>這個trait。

## Solution

為Reservation 實現FromRow<'r, PgRow>這個trait，並且實作from_row()這個方法，這個方法是用來將資料庫的資料轉換成也就是mapped struct的方法。
```rust
impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let range: PgRange<DateTime<Utc>> = row.get("timespan");
        let range: NativeRange<DateTime<Utc>> = range.into();

        let start = range.start.unwrap();
        let end = range.end.unwrap();

        Ok(Self {
            id: row.get::<Uuid, _>("id").to_string(),
            user_id: row.get("user_id"),
            resource_id: row.get("resource_id"),
            start: Some(convert_to_timestamp(start)),
            end: Some(convert_to_timestamp(end)),
            note: row.get("note"),
            status: ReservationStatus::from(row.get::<RsvpStatus, _>("status")) as i32,
        })
    }
}
```

## ERROR：stack overflow
### Describe:
我在為我的Error枚舉實現PartialEq時遇到了問題。我的最初的實現看起來像這樣：
```rust
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DbError(_), Self::DbError(_)) => true,
            (v1, v2) => v1 == v2,
        }
    }
}
```
我的程式發生了stack overflow。這是因為在`(v1, v2) => v1 == v2`這行，當v1和v2是具有相同內部結構的Error變體（例如InvalidUserId或InvalidReservation），這一行會再次呼叫eq方法，造成無窮遞迴。

## Error Analysis

我了解到問題出在我通用的比較處理上。我想要的是比較每個變體的內部值，而不是再次呼叫整個枚舉的eq方法。也就是說假設我現在有兩個InvalidUserId，我想要比較此時他們會呼叫這行`(v1, v2) => v1 == v2`,問題尷尬就尷尬在後面要執行時，會再次呼叫`eq`方法，造成無窮遞迴。

## Solution

我修改了PartialEq的實現，明確地處理每個變體，並對它們的內部值進行比較。這避免了遞迴問題：
```rust
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DbError(_), Self::DbError(_)) => true,
            (Self::InvalidTime, Self::InvalidTime) => true,
            (Self::InvalidUserId(v1), Self::InvalidUserId(v2)) => v1 == v2,
            // 其他變體的處理...
            _ => false,
        }
    }
}
```
通過明確地比較每個變體的內部值，而不是再次呼叫枚舉的eq方法，例如，假設`InvalidResourceId`變體內部存儲的是`String`類型，那麼`v1 == v2`將呼叫String的`PartialEq`方法，而不是自定義的Error枚舉的PartialEq方法。這就避免了無窮遞迴的問題，因為它不會再次進入Error枚舉的PartialEq實現。

## ERROR：rust interactive with database has some type errors
### Describe:
這是當我想要明明有在database中的參數列表預設page、page_size為1、10，但是當我進行rust端的testcase時，並且在我沒有給值的情況下，發現rust在進行query並不會將我原本寫好在database中的預設值帶入，導致我在測試test_query_should_return_vec_of_reservation發生以下錯誤：
```rust
ReservationQuery { resource_id: "", user_id: "yangid", status: Pending, start: Some(Timestamp { seconds: 1635750000, nanos: 0 }), end: Some(Timestamp { seconds: 1703995200, nanos: 0 }), page: 0, page_size: 0, desc: false }
test manager::tests::test_query_should_return_vec_of_reservation ... FAILED
```

## Error Analysis

關於page和page_size的預設值問題，我注意到在ReservationQuery結構中這兩個字段都有一個#[builder(setter(into), default)]屬性。這意味著，如果在構建此結構時沒有明確設置這兩個字段的值，它們將採用`預設值`。Rust中的i32型別的預設值為`0`。
因為：
```rust
    /// current page for the query
    #[prost(int32, tag = "6")]
    #[builder(setter(into), default)]
    pub page: i32,
    /// page size for the query
    #[prost(int32, tag = "7")]
    #[builder(setter(into), default)]
    pub page_size: i32,
```

所以，當我使用ReservationQueryBuilder::default()來創建一個新的查詢時，page和page_size的值都將為0，除非我明確地設置了它們的值。

## Solution

我在rust query進入資料庫調用query函數的時候直接給予page和page_size預設值的判斷，像是：
```sql
    -- if page_size is not between 10 and 100, set it to 10
    IF page_size < 10 OR page_size > 100 THEN
        page_size := 10;
    END IF;
    -- if page is less than 1, set it to 1
    IF page < 1 THEN
        page := 1;
    END IF;
```
就可以完美解決。

## ERROR： during the search database has performance's problem
### Describe:
當我在使用query function 進行pagination search data的時候，我發現由於使用`Limit`跟`Offset`進行查詢的時候，會將整個資料庫的資料都撈出來，然後再進行分頁，這樣的效能是非常差的。

## Error Analysis

使用OFFSET和LIMIT進行深分頁（即頁碼很大的情況）可能會對資料庫效能造成嚴重影響。以下是這種方法的主要問題：

- 過度掃描：正如剛剛提到的，當使用大的OFFSET值時，資料庫必須先掃描並跳過指定數量的紀錄，然後再返回所需的紀錄。這意味著資料庫實際上已經`掃描了比最終結果更多的紀錄`。隨著OFFSET值的增加，這種情況會變得更糟。

- 不一致性：使用OFFSET和LIMIT進行分頁可能會導致重複或遺失的紀錄，特別是當資料正在被更新或插入時。

- 難以維護：隨著數據量的增長，分頁的效能會進一步下降。

## Solution
