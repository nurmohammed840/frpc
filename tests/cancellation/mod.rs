use frpc::*;

async fn sleep_for_eternity() {
    std::future::pending::<()>().await;
}

fn stream_sleep_for_eternity() -> impl Output {
    sse!({
        yield "Going to sleep for eternity";
        std::future::pending::<()>().await;
    })
}

frpc::declare! {
    pub service Cancellation {
        rpc sleep_for_eternity = 1;
        rpc stream_sleep_for_eternity = 2;
    }
}
