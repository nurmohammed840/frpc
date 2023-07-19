use frpc::*;
use std::time::Duration;
use tokio::time::sleep;

const MSG: &str = "deadline passed!";

async fn unary_call(deadline: u8) -> Result<(), &'static str> {
    sleep(Duration::from_secs(deadline.into())).await;
    Err(MSG)
}

fn stream(deadline: u8) -> impl Output {
    sse!({
        yield format!("waiting for {deadline} secs");
        sleep(Duration::from_secs(deadline.into())).await;
        yield format!("timeout!");
        Result::<(), _>::Err(MSG)
    })
}

frpc::declare! {
    pub service Cancellation {
        rpc unary_call = 1;
        rpc stream = 2;
    }
}
