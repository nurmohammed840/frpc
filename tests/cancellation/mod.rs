use frpc::*;
use std::future::pending;

async fn sleep_for_eternity() {
    let _: () = pending().await;
}

fn stream_sleep_for_eternity() -> impl Output {
    sse!({
        yield "Going to sleep for eternity";
        let _: () = pending().await;
    })
}

frpc::declare! {
    pub service Cancellation {
        rpc sleep_for_eternity = 1;
        rpc stream_sleep_for_eternity = 2;
    }
}
