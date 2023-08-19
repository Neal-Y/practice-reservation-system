use std::process::Command;

use tonic_build::Builder;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .with_sql_type(&["reservation.ReservationStatus"])
        .with_builder(&["reservation.ReservationQuery", "reservation.FilterById"])
        .with_option_builder("reservation.ReservationQuery", &["start", "end"])
        .with_into_builder(
            "reservation.ReservationQuery",
            &[
                "user_id",
                "resource_id",
                "status",
                "page",
                "page_size",
                "desc",
            ],
        )
        .with_into_builder(
            "reservation.FilterById",
            &[
                "user_id",
                "resource_id",
                "status",
                "cursor",
                "page_size",
                "desc",
            ],
        )
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    Command::new("cargo").args(["fmt"]).output().unwrap();
}

/*
這段程式碼是用來使用 tonic_build crate 來將 protobuf 檔案編譯成 Rust 程式碼。
tonic 是一個 Rust 的 gRPC 框架，而 tonic_build 是 tonic 的一個構建工具，可以幫助開發人員將 protobuf 檔案轉換成 Rust 程式碼。

configure().out_dir("src/pb"): 這裡設定編譯出的 Rust 檔案應該被放在哪個目錄。這裡設定的是 "src/pb"。

type_attribute("reservation.ReservationStatus", "#[derive(sqlx::Type)]"): 這裡對編譯出的特定型別增加額外的屬性。
在這裡，我們對 "reservation.ReservationStatus" 這個型別增加了 #[derive(sqlx::Type)] 這個屬性，//?使得這個型別可以被 sqlx crate 作為一個資料庫型別來使用。

setter(into) 的部分意味著設定器方法可以接受任何可以被轉換為欄位型別的值。舉例來說，假設你有一個型別為 String 的欄位，但你想要也能夠接受 &str 作為輸入，into 屬性讓你可以這麼做，因為 &str 可以轉換為 String。

strip_option 是專為 Option<T> 型別的欄位而設。如果一個欄位是 Option<String>，你通常會這樣設置它：.field_name(Some("value".to_string()))。但如果你在這個欄位上使用了 strip_option，你可以直接這麼做：.field_name("value")，它內部會自動將 "value" 包裝成 Some("value".to_string())。

compile(&["protos/reservation.proto"], &["protos"]): 這裡是編譯 protobuf 檔案的地方。參數分別是 protobuf 檔案的路徑和包含 protobuf 檔案的目錄的路徑。

unwrap() 方法則是在編譯失敗時讓程式 panic，這在 main 函數中是可以接受的，因為如果編譯失敗，我們希望整個程式都終止。

所以整體來看，這段程式碼的目的是將 protobuf 檔案編譯成 Rust 程式碼，並對編譯出的特定型別增加一些額外的屬性。
*/

trait BuilderUtils {
    fn with_sql_type(self, path: &[&str]) -> Self;
    fn with_builder(self, path: &[&str]) -> Self;
    fn with_option_builder(self, path: &str, fields: &[&str]) -> Self;
    fn with_into_builder(self, path: &str, fields: &[&str]) -> Self;
}

impl BuilderUtils for Builder {
    fn with_sql_type(self, path: &[&str]) -> Self {
        path.iter().fold(self, |acc, path| {
            acc.type_attribute(path, "#[derive(sqlx::Type)]")
        })
    }
    fn with_builder(self, path: &[&str]) -> Self {
        path.iter().fold(self, |acc, path| {
            acc.type_attribute(path, "#[derive(derive_builder::Builder)]")
        })
    }
    // options for start and end
    fn with_option_builder(self, path: &str, fields: &[&str]) -> Self {
        fields.iter().fold(self, |acc, field| {
            acc.field_attribute(
                format!("{}.{}", path, field),
                "#[builder(setter(into, strip_option))]",
            )
        })
    }
    // turn into lots of fields
    fn with_into_builder(self, path: &str, fields: &[&str]) -> Self {
        fields.iter().fold(self, |acc, field| {
            acc.field_attribute(
                format!("{}.{}", path, field),
                "#[builder(setter(into),default)]",
            )
        })
    }
}
