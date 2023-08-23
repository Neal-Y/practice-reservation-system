### reservation core的專案

希望使用`gRPC`並且搭配著`protobuf`

- abi：
    - 這裡放的是一些型別和驗證，以及protobuf的檔案
- reservation：
    - 這裡放的是主要的業務邏輯，像是主要的reserve、change_status、delete等function
- service：
    - 這裡放的是對外的gRPC interface，把吃進來的protobuf定義的資料轉成reservation core的資料再將他們輸出出去即可。

        `輸入->校驗->轉換(required args by reservation core)->處理->轉換(gRPC interface)->輸出。`

    - 而作為監聽port的請求，透露給外面的就是service的gRPC server。


### 為什麼要這樣分？ 高內聚 (High Cohesion) and 低耦合 (Low Coupling)

1. *分層架構*：將業務邏輯 (reservation)、通訊協定和型別 (abi)、以及外部的 gRPC 伺服器接口分開，是在網路上查到一個很好的分層方式。這種設計方式使得代碼更容易維護、測試和擴展。

2. *業務邏輯 (reservation)*：如預訂、更改狀態、刪除等功能，放在這一部分。這樣做是因為這些都是業務邏輯的核心。

3. *通訊協定和型別 (abi)*：這個 crate 包含 protobuf 定義和相關的型別。將它們從核心業務邏輯中分離出來，有助於保持業務邏輯的清晰性，並使得在未來需要更改或擴展通訊協定時更容易進行。

4. *gRPC 伺服器接口*：這部分的工作主要是將客戶端的請求轉換成業務邏輯可以理解的請求，並將業務邏輯的回應轉換成 gRPC 接口的回應。你的流程：輸入 -> 校驗 -> 轉換 -> 處理 -> 轉換 -> 輸出。
