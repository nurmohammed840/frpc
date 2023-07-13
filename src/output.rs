use super::*;
use async_gen::futures_core::future::BoxFuture;
use std::{io::Result, marker::PhantomData};

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
    fn produce<'fut, 'cursor, 'data, 'transport, State, Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + Send + 'fut,
        state: State,
        cursor: &'cursor mut &'data [u8],
        transport: &'transport mut (impl Transport + Send),
    ) -> BoxFuture<'fut, ()>
    where
        'cursor: 'fut,
        'transport: 'fut,

        Self: 'fut,
        State: 'fut + Send,
        Args: 'fut + input::Input<'data, State> + Send;
}

impl<T> Output for Return<T>
where
    T: Send + Encode + TypeId,
{
    fn produce<'fut, 'cursor, 'data, 'transport, State, Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + Send + 'fut,
        state: State,
        cursor: &'cursor mut &'data [u8],
        transport: &'transport mut (impl Transport + Send),
    ) -> BoxFuture<'fut, ()>
    where
        'cursor: 'fut,
        'transport: 'fut,

        Self: 'fut,
        State: 'fut + Send,
        Args: 'fut + input::Input<'data, State>,
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

/// # What is wrong with this code ?
///
/// ```txt
/// let mut state = match Args::decode(state, reader) {
///     Ok(args) => Ok(func.call_once(args)),
///     Err(error) => Err(Some(io::Error::new(io::ErrorKind::InvalidInput, error))),
/// };
/// transport.unary(move |cx, buf| match state {
///     Ok(ref mut fut) => unsafe { Pin::new_unchecked(fut) }
///         .poll(cx)
///         .map(|data| Encode::encode::<{ crate::DATABUF_CONFIG }>(&data, buf)),
///     Err(ref mut err) => {
///         Poll::Ready(Err(err.take().expect("unary()` polled after completion")))
///     }
/// })
/// ```
///
/// I am assuming is that, Beacouse `Args::decode(...)` is outside,
/// The future is created outside on the stack, And then move into `transport.unary(|..| {})` closure.
/// So if Future is large then `Output::produce()` function also become large.
///
/// If `Output::produce()` is large then, various `Output::produce()` will create large future.
/// which is expensive to move around.
///
/// ```txt
/// // The size of this return future will be the largest size of `Output::produce(..)`
/// async fn execute(id, ...) { // in this case, it's `4kb`
///     match id {
///         1 => Output::produce(...).await, // Lets say it is create `64 kb` future
///         2 => Output::produce(...).await, // And this create `4kb` future
///         ...
///     }
/// }
/// ```
///
/// A solution is to wrap this function with `Box`.
/// But `transport.unary(..)` internally uses `Box` to wrap the future.
/// So let's take advantage of it.
///
/// Another adventage is that is will make function arguments parsing lazy,
/// If transport layer has any error, Which in result the future will never get polled.
enum FutState<'cursor, 'data, Func, State, Args, Fut>
where
    Func: std_lib::FnOnce<Args, Output = Fut>,
{
    Init {
        func: Func,
        state: State,
        cursor: &'cursor mut &'data [u8],
        _args: std::marker::PhantomData<Args>,
    },
    Poll(Fut),
    Done,
}

impl<'cursor, 'data, Func, State, Args, Fut> FutState<'cursor, 'data, Func, State, Args, Fut>
where
    Func: std_lib::FnOnce<Args, Output = Fut>,
    Args: input::Input<'data, State>,
{
    fn new(func: Func, state: State, cursor: &'cursor mut &'data [u8]) -> Self {
        FutState::Init {
            func,
            state,
            cursor,
            _args: PhantomData,
        }
    }

    fn poll<T>(&mut self, cb: impl FnOnce(&mut Fut) -> Poll<Result<T>>) -> Poll<Result<T>> {
        loop {
            match self {
                FutState::Init { .. } => match std::mem::replace(self, FutState::Done) {
                    FutState::Init {
                        state,
                        cursor,
                        func,
                        ..
                    } => match Args::decode(state, cursor) {
                        // This is the only place where we move this future.
                        // From now on we promise we will never move it again!
                        Ok(args) => *self = FutState::Poll(func.call_once(args)),
                        Err(err) => {
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                err,
                            )));
                        }
                    },
                    _ => unsafe { std::hint::unreachable_unchecked() },
                },
                FutState::Poll(ref mut fut) => return cb(fut),
                FutState::Done => panic!("`Output::produce` polled after completion"),
            }
        }
    }
}

impl<Fut> Output for Fut
where
    Fut: Future + Send,
    Fut::Output: Encode + TypeId,
{
    fn produce<'fut, 'cursor, 'data, 'transport, State, Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + Send + 'fut,
        state: State,
        cursor: &'cursor mut &'data [u8],
        transport: &'transport mut (impl Transport + Send),
    ) -> BoxFuture<'fut, ()>
    where
        'cursor: 'fut,
        'transport: 'fut,

        Self: 'fut,
        State: 'fut + Send,
        Args: 'fut + input::Input<'data, State> + Send,
    {
        let mut fut_state = FutState::new(func, state, cursor);
        transport.unary(move |cx, buf| {
            fut_state.poll(|fut| {
                unsafe { Pin::new_unchecked(fut) }
                    .poll(cx)
                    .map(|data| Encode::encode::<{ crate::DATABUF_CONFIG }>(&data, buf))
            })
        })
    }
}

impl<G> Output for SSE<G>
where
    G: AsyncGenerator + Send,
{
    fn produce<'fut, 'cursor, 'data, 'transport, State, Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + Send + 'fut,
        state: State,
        cursor: &'cursor mut &'data [u8],
        transport: &'transport mut (impl Transport + Send),
    ) -> BoxFuture<'fut, ()>
    where
        'cursor: 'fut,
        'transport: 'fut,

        Self: 'fut,
        State: 'fut + Send,
        Args: 'fut + input::Input<'data, State> + Send,
    {
        let mut fut_state = FutState::new(func, state, cursor);
        transport.server_stream(move |cx, buf| {
            fut_state.poll(|this| {
                unsafe { Pin::new_unchecked(&mut this.0) }
                    .poll_resume(cx)
                    .map(|gen_state| match gen_state {
                        GeneratorState::Yielded(val) => {
                            Encode::encode::<{ crate::DATABUF_CONFIG }>(&val, buf).map(|()| false)
                        }
                        GeneratorState::Complete(val) => {
                            Encode::encode::<{ crate::DATABUF_CONFIG }>(&val, buf).map(|()| true)
                        }
                    })
            })
        })
    }
}
