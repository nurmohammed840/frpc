use frpc::{databuf::Decode, Return};
use std::{io::Write, time::Instant};

const ITER: u32 = 10000000;

#[tokio::main]
async fn main() {
    let time = Instant::now();
    let left = normal().await;
    println!("Normal: {:?}", time.elapsed());

    let time = Instant::now();
    let right = rpc().await;
    println!("RPC: {:?}", time.elapsed());

    assert_eq!(left, right)
}

#[inline(never)]
fn add(a: u8, b: u8) -> Return<u8> {
    Return(a.wrapping_add(b))
}

#[inline(never)]
async fn normal() -> Vec<u8> {
    let mut tcp = vec![];
    for i in 0..ITER {
        let data = [i as u8, i as u8];
        let mut cursor = data.as_slice();

        // Why `.write_all()` instade of `.push()` ?
        // Ans: transport layer usually don't have push method.

        tcp.write_all(&[Box::pin(async {
            // Why `Box::pin(...)` ? see [FutState] in `./src/output.rs` for more.

            // Why `Decode::decode()` ?
            // Ans: Some sort of decoding in needed in any protocol.
            let a = Decode::decode::<{ frpc::DATABUF_CONFIG }>(&mut cursor).unwrap();
            let b = Decode::decode::<{ frpc::DATABUF_CONFIG }>(&mut cursor).unwrap();

            add(a, b).0
        })
        .await])
            .unwrap();
    }
    tcp
}

#[inline(never)]
async fn rpc() -> Vec<u8> {
    let mut tcp = DummyTransport(vec![]);
    for i in 0..ITER {
        let data = [i as u8, i as u8];
        let mut cursor = data.as_slice();
        frpc::Output::produce(add, (), &mut cursor, &mut tcp).await
    }
    tcp.0
}

struct DummyTransport(Vec<u8>);

impl frpc::Transport for DummyTransport {
    async fn unary_sync(
        &mut self,
        cb: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()> + Send,
    ) {
        cb(&mut self.0).unwrap()
    }

    async fn unary(
        &mut self,
        _poll: impl FnMut(
                &mut std::task::Context,
                &mut dyn std::io::Write,
            ) -> std::task::Poll<std::io::Result<()>>
            + Send,
    ) {
        todo!()
    }

    async fn server_stream(
        &mut self,
        _poll: impl FnMut(
                &mut std::task::Context,
                &mut dyn std::io::Write,
            ) -> std::task::Poll<std::io::Result<bool>>
            + Send,
    ) {
        todo!()
    }
}
