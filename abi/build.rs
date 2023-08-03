fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .type_attribute("reservation.ReservationStatus", "#[derive(sqlx::Type)]")
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();
}

/*
這段程式碼是用來使用 tonic_build crate 來將 protobuf 檔案編譯成 Rust 程式碼。
tonic 是一個 Rust 的 gRPC 框架，而 tonic_build 是 tonic 的一個構建工具，可以幫助開發人員將 protobuf 檔案轉換成 Rust 程式碼。

configure().out_dir("src/pb"): 這裡設定編譯出的 Rust 檔案應該被放在哪個目錄。這裡設定的是 "src/pb"。

type_attribute("reservation.ReservationStatus", "#[derive(sqlx::Type)]"): 這裡對編譯出的特定型別增加額外的屬性。
在這裡，我們對 "reservation.ReservationStatus" 這個型別增加了 #[derive(sqlx::Type)] 這個屬性，//?使得這個型別可以被 sqlx crate 作為一個資料庫型別來使用。

compile(&["protos/reservation.proto"], &["protos"]): 這裡是編譯 protobuf 檔案的地方。參數分別是 protobuf 檔案的路徑和包含 protobuf 檔案的目錄的路徑。

unwrap() 方法則是在編譯失敗時讓程式 panic，這在 main 函數中是可以接受的，因為如果編譯失敗，我們希望整個程式都終止。

所以整體來看，這段程式碼的目的是將 protobuf 檔案編譯成 Rust 程式碼，並對編譯出的特定型別增加一些額外的屬性。
*/
