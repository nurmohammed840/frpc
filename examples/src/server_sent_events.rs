use frpc::*;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Output)]
struct Event {
    elapsed: u64,
}

fn get_events(count: u8) -> impl Output {
    sse! {
        if count > 10 {
            return Err(format!("count: {count} should be <= 10"));
        }
        let time = Instant::now();
        for _ in 0..count {
            sleep(Duration::from_secs(1)).await;
            yield Event { elapsed: time.elapsed().as_secs() }
        }
        Ok(())
    }
}

declare! {
    pub service ServerSentEvents {
        rpc get_events = 1;
    }
}
