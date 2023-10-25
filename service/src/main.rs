use abi::Config;
use reservation_service::start_the_server;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 可能的路徑直接在這邊加上就好
    let possible_path = [
        String::from("./reservation.yml"),
        String::from("/etc/reservation.yml"),
        shellexpand::tilde("~/.config/reservation.yml").into_owned(),
    ];

    // 這邊是讀取環境變數，如果沒有的話就用上面的路徑
    let filepath = std::env::var("RESERVATION_CONFIG").unwrap_or_else(|_| {
        //? 閉包可以捕捉外部的變數，所以可以直接用上面的路徑
        possible_path
            .into_iter()
            .find(|path| Path::new(path).exists())
            .ok_or_else(|| abi::Error::NotFound)
            .unwrap()
    });

    let config = Config::load(filepath)?;
    start_the_server(&config).await
}

/*

所有關於gRPC的reservation程式碼，到了最後就是將那些程式碼透過這個gRPC的向外部提供服務。

gRPC 是一個高性能、開源和通用的 RPC (遠程過程呼叫) 框架，它使伺服器和客戶端之間的通信變得更簡單、更快速。在先前的 reservation 程式碼中

定義了伺服器應該如何處理請求、數據結構是什麼，以及可能的錯誤情況是什麼。

但為了讓其他服務或客戶端實際上能夠呼叫這些功能，我就需要啟動一個伺服器來“監聽”來自外部的請求。這正是 main 函數所做的事情。

這個 main 函數：

加載配置文件。
創建之前定義的 RsvpService。
將該服務添加到 gRPC 伺服器中。
在指定的地址上啟動伺服器，以便開始接收和響應外部的請求。
因此，這個 main 函數的主要工作是將先前寫的 gRPC 的 reservation 程式碼向外部公開，使其他服務和客戶端可以與它互動。

*/
