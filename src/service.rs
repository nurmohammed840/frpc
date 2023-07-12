//! Alternative service API
//!
//! ### Down Side
//!
//! - Runtime is required to initialize functions (no compile time check).
//! - There is some runtime cost (hashing + dynamic dispatch) for calling functions.
//! - More verbose then `declare!` macro
//! - ...

use super::*;
use crate::{
    input::Input,
    output::{AsyncWriter, Output},
};
use std::{collections::HashMap, future::Future, marker::PhantomData, pin::Pin};

type RpcFutute<'a, Output = std::io::Result<()>> =
    Pin<Box<dyn Future<Output = Output> + Send + 'a>>;

trait RpcHandler<State, AsyncWriter> {
    fn call<'w>(&self, state: State, data: Box<[u8]>, _: &'w mut AsyncWriter) -> RpcFutute<'w>;
}

struct Handler<Func, Args> {
    func: Func,
    _phantom_args: PhantomData<Args>,
}

impl<Func, Args> From<Func> for Handler<Func, Args> {
    fn from(func: Func) -> Self {
        Self {
            func,
            _phantom_args: PhantomData,
        }
    }
}

impl<State, Writer, Func, Args, Ret> RpcHandler<State, Writer> for Handler<Func, Args>
where
    Func: fn_once::FnOnce<Args, Output = Ret> + Clone,
    Args: for<'de> Input<'de, State>,
    Ret: Output + 'static,
    Writer: AsyncWriter + Unpin + Send,
{
    fn call<'w>(&self, state: State, data: Box<[u8]>, w: &'w mut Writer) -> RpcFutute<'w> {
        let mut reader = &*data;
        let args = Args::decode(state, &mut reader).unwrap();
        let output = self.func.clone().call_once(args);
        Output::send_output(output, w)
    }
}

pub struct Service<Writer, State = ()> {
    handlers: HashMap<u16, Box<dyn RpcHandler<State, Writer>>>,
}

impl<Writer, State> Service<Writer, State>
where
    Writer: AsyncWriter + Unpin + Send,
{
    pub fn new() -> Self {
        Self {
            handlers: Default::default(),
        }
    }

    pub fn rpc<Func, Args, Ret>(mut self, id: u16, _name: &str, func: Func) -> Self
    where
        Func: fn_once::FnOnce<Args, Output = Ret> + Clone + 'static,
        Args: for<'de> Input<'de, State> + 'static,
        Ret: Future + Send + 'static,
        Ret::Output: databuf::Encode,
    {
        self.handlers.insert(id, Box::new(Handler::from(func)));
        self
    }

    pub fn call<'a>(
        &self,
        state: State,
        id: u16,
        data: Box<[u8]>,
        w: &'a mut Writer,
    ) -> Option<RpcFutute<'a>> {
        Some(self.handlers.get(&id)?.call(state, data, w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn foo(_: State<()>) {}
    async fn get_num() -> &'static str {
        "42"
    }

    #[tokio::test]
    async fn run() {
        let service = Service::new()
            .rpc(1, "foo", foo)
            .rpc(2, "get_num", get_num)
            .rpc(3, "bar", || async {});

        let mut output = vec![];
        let _ = service
            .call((), 2, Box::new([]), &mut output)
            .expect("no function found")
            .await;

        assert_eq!(output, [2, b'4', b'2']);
    }
}
