use frpc_transport_core::{BoxFuture, Transport};
use h2x::http::StatusCode;
pub use h2x::*;
use std::{
    fmt::Debug,
    future::poll_fn,
    io, mem, ptr,
    task::{Context, Poll},
};

#[derive(Debug, Clone)]
pub struct Config {
    pub max_unary_payload_size: u32,
}
impl Config {
    pub const fn new() -> Self {
        Self {
            max_unary_payload_size: 128 * 1024,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Ctx {
    pub req: Request,
    pub res: Response,
}

use frpc_transport_core::Executor;

impl Ctx {
    pub fn new(req: Request, res: Response) -> Self {
        Self { req, res }
    }

    pub async fn serve_v2<S, E>(&mut self, conf: &Config, state: S, _: E) -> StatusCode
    where
        E: Executor<State = S>,
    {
        match self.req.headers.get("content-length") {
            Some(len) => {
                let Ok(Ok(len)) = len.to_str().map(str::parse::<u32>) else {
                    return StatusCode::BAD_REQUEST;
                };
                if len > conf.max_unary_payload_size {
                    return StatusCode::PAYLOAD_TOO_LARGE;
                }
                let mut buf = Vec::with_capacity(len as usize);
                while let Some(bytes) = self.req.data().await {
                    let Ok(bytes) = bytes else {
                        return StatusCode::PARTIAL_CONTENT;
                    };
                    buf.extend_from_slice(&bytes);
                    if buf.len() > len as usize {
                        return StatusCode::PARTIAL_CONTENT;
                    }
                }
                if buf.len() < 2 {
                    return StatusCode::BAD_REQUEST;
                }
                let id = u16::from_le_bytes([buf[0], buf[1]]);
                let data = &buf[2..];

                let mut transport = RpcResponder(&mut self.res);
                let mut cursor = data;
                let Some(fut) = E::execute(state, id, &mut cursor, &mut transport) else {
                    return StatusCode::NOT_FOUND;
                };
                fut.await;
                StatusCode::OK
            }
            None => {
                // Uni-Stream, Bi-Stream
                StatusCode::NOT_IMPLEMENTED
            }
        }
    }

    pub async fn serve<'this, State>(
        &'this mut self,
        conf: &Config,
        state: State,
        // TODO: We should use a trait here.
        executor: impl for<'fut> FnOnce(
            State,
            u16,
            &'fut mut &[u8],
            &'fut mut RpcResponder<'this>,
        ) -> Option<BoxFuture<'fut, ()>>,
    ) -> StatusCode {
        match self.req.headers.get("content-length") {
            Some(len) => {
                let Ok(Ok(len)) = len.to_str().map(str::parse::<u32>) else {
                    return StatusCode::BAD_REQUEST;
                };
                if len > conf.max_unary_payload_size {
                    return StatusCode::PAYLOAD_TOO_LARGE;
                }
                let mut buf = Vec::with_capacity(len as usize);
                while let Some(bytes) = self.req.data().await {
                    let Ok(bytes) = bytes else {
                        return StatusCode::PARTIAL_CONTENT;
                    };
                    buf.extend_from_slice(&bytes);
                    if buf.len() > len as usize {
                        return StatusCode::PARTIAL_CONTENT;
                    }
                }
                if buf.len() < 2 {
                    return StatusCode::BAD_REQUEST;
                }
                let id = u16::from_le_bytes([buf[0], buf[1]]);
                let data = &buf[2..];

                let mut transport = RpcResponder(&mut self.res);
                let mut cursor = data;
                let Some(fut) = executor(state, id, &mut cursor, &mut transport) else {
                    return StatusCode::NOT_FOUND;
                };
                fut.await;
                StatusCode::OK
            }
            None => {
                // Uni-Stream, Bi-Stream
                StatusCode::NOT_IMPLEMENTED
            }
        }
    }
}

pub struct RpcResponder<'a>(&'a mut Response);

impl Transport for RpcResponder<'_> {
    async fn unary_sync(&mut self, cb: impl FnOnce(&mut dyn io::Write) -> io::Result<()> + Send) {
        let mut cb = Some(cb);
        self.unary(move |_, buf| {
            Poll::Ready(match cb.take() {
                Some(cb) => cb(buf),
                None => unreachable!(),
            })
        })
        .await
    }

    async fn unary(
        &mut self,
        mut poll: impl FnMut(&mut Context, &mut dyn io::Write) -> Poll<io::Result<()>> + Send,
    ) {
        let mut response = http::Response::new(());
        *response.headers_mut() = mem::take(&mut self.0.headers);

        let mut buf = vec![];

        if let Some(output) = poll_fn(|cx| match self.0.sender.poll_reset(cx) {
            Poll::Ready(_reset_reason) => Poll::Ready(None),
            Poll::Pending => poll(cx, &mut buf).map(Some),
        })
        .await
        {
            match output {
                Ok(()) => {
                    let is_empty = buf.is_empty();
                    if let Ok(stream) = self.0.sender.send_response(response, is_empty) {
                        if !is_empty {
                            let _ = h2x::Responder { inner: stream }
                                .write_bytes(buf.into(), true)
                                .await;
                        }
                    }
                }
                Err(_parse_err) => {
                    // dbg!(_parse_err);
                    *response.status_mut() = StatusCode::NOT_ACCEPTABLE;
                    let _ = self.0.sender.send_response(response, true);
                }
            }
        }
    }

    async fn server_stream(
        &mut self,
        mut poll: impl FnMut(&mut Context, &mut dyn io::Write) -> Poll<io::Result<bool>> + Send,
    ) {
        let mut response = http::Response::new(());
        *response.headers_mut() = mem::take(&mut self.0.headers);

        let Ok(inner) = self.0.sender.send_response(response, false) else {
            return;
        };

        let mut stream = h2x::Responder { inner };
        let mut buf = vec![0; 4];

        while let Ok(done) = poll_fn(|cx| poll(cx, &mut buf)).await {
            let len = buf.len() - 4;
            if len >= (1 << 31) {
                break;
            }
            unsafe {
                let len = (len as u32).to_le_bytes();
                // SAFETY: `buf` is valid for `4` bytes.
                ptr::copy_nonoverlapping(len.as_ptr(), buf.as_mut_ptr(), 4);
            }
            match done {
                false => {
                    if len == 0 {
                        continue;
                    }
                    let bytes = std::mem::replace(&mut buf, vec![0; 4]);
                    if stream.write_bytes(bytes.into(), false).await.is_err() {
                        break;
                    }
                }
                true => {
                    buf[3] |= 0b1000_0000;
                    let _ = stream.write_bytes(buf.into(), true).await;
                    break;
                }
            }
        }
    }
}
