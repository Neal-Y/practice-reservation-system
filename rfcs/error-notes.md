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

使用cursor以及相關演算法來解決這個問題，我在這裡使用了`keyset pagination`，這是一種基於資料庫索引的分頁方法。它使用資料庫索引中的值來決定下一頁的起始點，而不是使用OFFSET。這種方法的好處是，它可以保證分頁的效能不會隨著數據量的增長而下降。

## ERROR： during write the gRPC server's test case has some problem
### Describe:
問題出現在的我想要對gRPC server進行測試，但是在我使用sqlx_database_tester時突然意識到這樣只會對資料庫進行測試，而不是gRPC server，這樣的測試是不完整的。

測試案例是直接對 RsvpService 進行操作，而不是模擬一個實際的gRPC請求到服務器。這意味著，我是在測試業務邏輯，但沒有測試gRPC服務器的整體行為。

簡單來說就是要integration test，而不是unit test。`測試一條龍服務運作如何。`

## Error Analysis

如果我希望完整地測試gRPC服務器，我需要做以下事情：

- 啟動gRPC服務器：我需要在測試案例中啟動gRPC服務器，可能需要指定一個短暫的端口來確保不會有端口衝突。
- 模擬gRPC客戶端：使用gRPC的Rust客戶端工具來模擬一個客戶端，並發送請求到服務器。
- 驗證服務器響應：當服務器響應後，確認你收到了預期的響應。

這樣，我就可以完整地模擬一個真實的gRPC請求和響應過程，並確保服務器行為是正確的。

`雖然我也想這樣做不過我更想focus在rust內部做integration tests`

## Solution

那根據我最後的決策，我想在測試時生成一個跟生產環境一模一樣的database供我做最貼近實際資料庫的測試，以下是我的步驟：

1. `初始化`：
    使用 lazy_static 創建一個全局的 tokio Runtime (TEST_RT)。這將在測試期間只初始化一次，並用於執行異步任務。

    定義 TestConfig 結構，它代表測試配置並提供以下功能：

    在初始化時，根據預設的配置文件 (config.yml) 創建一個具有唯一名稱的新測試數據庫。
    在被丟棄（Drop）時，刪除先前創建的測試數據庫。

2. `測試`：
    在 rpc_reserve_should_work 測試中，我執行了以下操作：
    - 使用 TestConfig 創建測試配置。
    - 使用此測試配置初始化 RsvpService 服務。
    - 創建一個新的預訂請求 (ReserveRequest)。
    - 調用 reserve 方法處理此請求。
    - 驗證回應中的預訂是否符合預期。

3. `清理`：
    當 TestConfig 實例超出作用域並被丟棄時，其 Drop 實現將被調用。在這個 Drop 實現中，執行了以下操作：
    - 斷開與該測試數據庫的所有連接。`發現新大陸(pg_database function)`
    - 刪除先前創建的測試數據庫。

## ERROR： in order to avoid the sql injection, format!() or .bind()
### Describe:
"parameterized queries" 或 "prepared statements"

r# 是一個原始字串文字 (raw string literal) 的開始標記。當你想表示一個字串，且這個字串中包含很多反斜線 ( \ ) 或者特殊字符時，使用原始字串可以讓你更輕鬆地表達這些內容，因為原始字串不會進行任何轉義。

## Error Analysis

首先，養成好習慣，當我需要使用`format!()`時因為避免裡面的符號被胡亂翻譯，所以我使用`r#""#`來包裝，像是：
```rust
sqlx::query(&format!(r#"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND datname = '{}'"#, dbname))
```
亦或是：

`普通字串文字`："Hello\nWorld" 會被解釋為兩行的 Hello 和 World，因為 \n 是一個換行字符。

`原始字串文字`：r#"Hello\nWorld"# 會被直接解釋為 `Hello\nWorld`。

但前提是確保團隊看得懂，同時我也要注意到，如果我是在`sqlx::query()`也就是在使用查詢語句的情況下使用`format!()`時，我必須要確保`sql injection`的問題。

像是在上面的例子中，假設dbname的值是`"'; DROP TABLE important_data; --"`,那麼該查詢將會試圖刪除名為important_data的表。

## Solution

- 最小權限原則:

    在建立資料庫連接的使用者時，不要授予該使用者「超級使用者」或「管理員」的角色。
    只給予需要的權限。例如，如果一個應用程序只需要從資料庫中讀取資料，那麼該使用者不應該有寫入或修改的權限。
    在大多數的RDBMS系統，如PostgreSQL或MySQL中，都可以使用GRANT和REVOKE語句來管理使用者權限。

- 編碼規範:

    始終使用參數化查詢或資料庫提供的API方法，而不是字符串拼接或format!()來創建SQL語句。
    例如，在sqlx庫中，可以使用`.bind()`方法來綁定參數。

- 程式碼審查:

    設定定期的程式碼審查流程，並確保團隊中有資深或具備安全知識的工程師參與。
    使用工具，如git進行版本控制，並在合併更改之前，通過合併請求（Merge Request）或拉取請求（Pull Request）進行審查。

- 使用專業工具:

    使用靜態應用程序安全測試(SAST)工具，這些工具可以自動掃描源代碼，尋找常見的安全問題，例如SQL注入、跨站腳本(XSS)等。
    使用動態應用程序安全測試(DAST)工具來執行實時的安全測試。
    使用資料庫活動監控工具，這些工具可以實時監控資料庫查詢，並在偵測到可疑或異常活動時發出警報。

## ERROR： PgDatabaseError: xxxexample is accessed by other users, there is one other session using the database.
### Describe:
這個是當我在創建與生產環境相同的的資料庫時，在測試完畢後想把他Drop時出現的問題，由於我在Drop時無database斷開所有連接，所以出現PgDatabaseError: xxxexample is accessed by other users, there is one other session using the database.`的錯誤。

## Error Analysis

```sql
thread '<unnamed>' panicked at 'Error while querying the drop database: Database(PgDatabaseError { severity: Error, code: "55006", message: "database \"test_7d9a76e7-9d7c-4454-9551-a6f072f4c295\" is being accessed by other users", detail: Some("There is 1 other session using the database."), hint: None, position: None, where: None, schema: None, table: None, column: None, data_type: None, constraint: None, file: Some("dbcommands.c"), line: Some(933), routine: Some("dropdb") })', service/src/service.rs:261:26
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread 'service::tests::rpc_reserve_should_work' panicked at 'failed to drop database: Any { .. }', service/src/service.rs:265:14
```

原因是因為我在手動Drop我的TestConfig中的DB時，資料庫還有其他線程連接著。

## Solution

這時候參考sqlx-database-tester crate的source code時，發現我可以在rust中操作資料庫層像是：

```rust
sqlx::query(&format!(r#"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND datname = '{}'"#, #database_name))
				.execute(&db_pool)
				.await
				.expect("Terminate all other connections");
```

`使他能夠在被Drop前，確保所有連線都被斷開，這樣就不會出現我剛剛提到的database error`

## ERROR： Determine whether it is necessary to extract the database test pre-work into a separate crate
### Describe:
just like title means.

## Error Analysis
just a concept that I need to think about it.

- `將其放在共用的模組中`：你可以將這些工具函數和結構放在一個共用的模組中，然後在需要的地方引入它。這可以保持你的代碼組織整潔，並避免重複。

- `使用 workspace`：如果你的系統實際上包含多個互相相依的 Rust 專案，你可以考慮使用 Cargo workspace。workspace 允許你在單一的頂層目錄下管理和建構多個相關的 Rust 專案。這樣，你可以在其中一個 crate 中定義這些工具功能，並在其他 crate 中引用它。

- `稍後再決定`：你也可以等待，看看隨著專案的發展，這些工具是否確實在多個地方被重用，或者是否出現了其他需要將其提取到獨立 crate 的原因。

## Solution
postpone my decision.

but in practice, if u need to keep the code private, u can try use the git repo which are not public, only your team can access it. that means u don't publish your own code to crate.io.
