use async_gen::futures_core::future::BoxFuture;

use super::*;

/// It represents the output of an rpc function.
///
/// User cannot implement this trait.
/// It is mainly used to infer the return type.
///
/// ## Example
///
/// ```
/// use frpc::*;
///
/// // same as: `fn async add(a: u32, b: u32) -> Result<u32, &'static str>;`
/// fn div(a: u32, b: u32) -> impl Output {
///     async move {
///         if b == 0 {
///             return Err("Cannot be divided by zero");
///         }
///         Ok(a / b)
///     }
/// }
/// ```
pub trait Output: crate::output_type::OutputType {
    #[doc(hidden)]
    fn produce<'data, State, Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + Send,
        state: State,
        cursor: &mut &'data [u8],
        transport: &mut (impl Transport + Send),
    ) -> impl Future<Output = ()> + Send
    where
        State: Send,
        Args: input::Input<'data, State> + Send;

    #[inline]
    #[doc(hidden)]
    fn _produce<'fut, 'cursor, 'data, 'transport, State, Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + Send + 'fut,
        state: State,
        cursor: &'cursor mut &'data [u8],
        transport: &'transport mut (impl Transport + Send),
    ) -> Option<BoxFuture<'fut, ()>>
    where
        Self: 'fut,
        'cursor: 'fut,
        'transport: 'fut,
        State: Send + 'fut,
        Args: input::Input<'data, State> + Send + 'fut,
    {
        Some(Box::pin(Self::produce(func, state, cursor, transport)))
    }
}

impl<T> Output for Return<T>
where
    T: Send + Encode + TypeId,
{
    fn produce<'data, State, Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + Send,
        state: State,
        cursor: &mut &'data [u8],
        transport: &mut (impl Transport + Send),
    ) -> impl Future<Output = ()> + Send
    where
        State: Send,
        Args: input::Input<'data, State> + Send,
    {
        transport.unary_sync(|buf| match Args::decode(state, cursor) {
            Ok(args) => {
                let this = func.call_once(args);
                Encode::encode::<{ crate::DATABUF_CONFIG }>(&this.0, buf)
            }
            Err(err) => Err(io::Error::new(io::ErrorKind::InvalidInput, err)),
        })
    }
}

impl<Fut> Output for Fut
where
    Fut: Future + Send,
    Fut::Output: Encode + TypeId,
{
    fn produce<'data, State, Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + Send,
        state: State,
        cursor: &mut &'data [u8],
        transport: &mut (impl Transport + Send),
    ) -> impl Future<Output = ()> + Send
    where
        State: Send,
        Args: input::Input<'data, State> + Send,
    {
        let mut state = match Args::decode(state, cursor) {
            Ok(args) => Ok(func.call_once(args)),
            Err(error) => Err(Some(io::Error::new(io::ErrorKind::InvalidInput, error))),
        };
        transport.unary(move |cx, buf| match state {
            Ok(ref mut fut) => unsafe { Pin::new_unchecked(fut) }
                .poll(cx)
                .map(|ret| Encode::encode::<{ crate::DATABUF_CONFIG }>(&ret, buf)),
            Err(ref mut err) => {
                Poll::Ready(Err(err.take().expect("Transport::unary(..)` polled after completion")))
            }
        })
    }
}

impl<G> Output for SSE<G>
where
    G: AsyncGenerator + Send,
{
    fn produce<'data, State, Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + Send,
        state: State,
        cursor: &mut &'data [u8],
        transport: &mut (impl Transport + Send),
    ) -> impl Future<Output = ()> + Send
    where
        State: Send,
        Args: input::Input<'data, State> + Send,
    {
        let mut state = match Args::decode(state, cursor) {
            Ok(args) => Ok(func.call_once(args)),
            Err(error) => Err(Some(io::Error::new(io::ErrorKind::InvalidInput, error))),
        };
        transport.server_stream(move |cx, buf| match state {
            Ok(ref mut async_generator) => unsafe { Pin::new_unchecked(&mut async_generator.0) }
                .poll_resume(cx)
                .map(|gen_state| match gen_state {
                    GeneratorState::Yielded(val) => {
                        Encode::encode::<{ crate::DATABUF_CONFIG }>(&val, buf).map(|()| false)
                    }
                    GeneratorState::Complete(val) => {
                        Encode::encode::<{ crate::DATABUF_CONFIG }>(&val, buf).map(|()| true)
                    }
                }),
            Err(ref mut err) => {
                Poll::Ready(Err(err.take().expect("Transport::server_stream(..)` polled after completion")))
            }
        })
    }
}
