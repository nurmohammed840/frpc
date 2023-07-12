use std::time::Duration;

use frpc::*;
use tokio::time::sleep;

fn say_hello_many_times(iter: u8, delay: u16) -> impl Output {
    sse!({
        for _ in 0..iter {
            yield "Hello!";
            sleep(Duration::from_millis(delay.into())).await;
        }
        return "bye!";
    })
}

declare! {
    /// [Server-Sent Events (SSE)](https://en.wikipedia.org/wiki/Server-sent_events) is a server push technology enabling
    /// a client to receive automatic updates from a server
    pub service ServerSentEvents {
        /// Say "Hello!" of specified number of times with a given
        /// delay (in milliseconds) between each message.
        rpc say_hello_many_times = 1;
    }
}
