use std::{
    // future::Future,
    io::{self, Result},
    task::{Context, Poll},
};

#[doc(hidden)]
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// It defines the behavior for sending responses over a transport channel.
pub trait Transport {
    fn unary_sync<'this, 'fut, CB>(&'this mut self, cb: CB) -> BoxFuture<'fut, ()>
    where
        'this: 'fut,
        Self: 'fut,
        CB: for<'buf> FnOnce(&'buf mut dyn io::Write) -> Result<()> + Send + 'fut;

    fn unary<'this, 'fut, P>(&'this mut self, poll: P) -> BoxFuture<'fut, ()>
    where
        'this: 'fut,
        Self: 'fut,
        P: Send + 'fut,
        P: for<'cx, 'w, 'buf> FnMut(
            &'cx mut Context<'w>,
            &'buf mut dyn io::Write,
        ) -> Poll<Result<()>>;

    fn server_stream<'this, 'fut, P>(&'this mut self, poll: P) -> BoxFuture<'fut, ()>
    where
        'this: 'fut,
        Self: 'fut,
        P: Send + 'fut,
        P: for<'cx, 'w, 'buf> FnMut(
            &'cx mut Context<'w>,
            &'buf mut dyn io::Write,
        ) -> Poll<Result<bool>>;
}

// pub trait TransportV2 {
//     fn unary_sync(
//         &mut self,
//         cb: impl FnOnce(&mut dyn io::Write) -> Result<()> + Send,
//     ) -> impl Future<Output = ()> + Send;

//     fn unary(
//         &mut self,
//         poll: impl FnMut(&mut Context, &mut dyn io::Write) -> Poll<Result<()>> + Send,
//     ) -> impl Future<Output = ()> + Send;

//     fn server_stream(
//         &mut self,
//         poll: impl FnMut(&mut Context, &mut dyn io::Write) -> Poll<Result<bool>> + Send,
//     ) -> impl Future<Output = ()> + Send;
// }
