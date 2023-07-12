use frpc::{sse, Output};

fn buffers() -> impl Output {
    sse!({
        for i in 1..1024 {
            yield vec![i as u8; i];
        }
        "Done!"
    })
}

fn chunks(count: u8, size: u32) -> impl Output {
    assert!(size < 32 * 1024 * 1024); // max size 32MB
    sse!({
        for i in 0..count {
            yield vec![i; size as usize];
        }
    })
}

frpc::declare! {
    pub service SSETest {
        rpc buffers = 1;
        rpc chunks  = 2;
    }
}
