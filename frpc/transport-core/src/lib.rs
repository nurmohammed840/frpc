use std::{
    future::Future,
    io::{self, Result},
    task::{Context, Poll},
};

#[doc(hidden)]
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

pub trait Transport {
    fn unary_sync(
        &mut self,
        cb: impl FnOnce(&mut dyn io::Write) -> Result<()> + Send,
    ) -> impl Future<Output = ()> + Send;

    fn unary(
        &mut self,
        poll: impl FnMut(&mut Context, &mut dyn io::Write) -> Poll<Result<()>> + Send,
    ) -> impl Future<Output = ()> + Send;

    fn server_stream(
        &mut self,
        poll: impl FnMut(&mut Context, &mut dyn io::Write) -> Poll<Result<bool>> + Send,
    ) -> impl Future<Output = ()> + Send;
}

#[doc(hidden)]
pub trait Executor {
    type State;
    fn execute<'fut, TR>(
        state: Self::State,
        id: u16,
        cursor: &'fut mut &[u8],
        transport: &'fut mut TR,
    ) -> Option<impl Future<Output = ()> + Send + 'fut>
    where
        TR: Transport + Send;
}

