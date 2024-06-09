#![allow(non_snake_case)]
use frpc::*;

/// The request message containing the user's name.
#[derive(Input)]
struct HelloRequest {
    name: String,
}

/// The response message containing the greetings.
#[derive(Output)]
struct HelloReply {
    message: String,
}

async fn SayHello(req: HelloRequest) -> HelloReply {
    HelloReply {
        message: format!("Hello {}", req.name),
    }
}

fn SayHelloAgain(req: HelloRequest) -> impl Output {
    async move {
        HelloReply {
            message: format!("Hello Again, {}", req.name),
        }
    }
}

declare! {
    /// The greeting service definition.
    pub service Greeter {
        /// Sends a greeting
        rpc SayHello = 1;

        /// Sends another greeting
        rpc SayHelloAgain = 2;
    }
}

impl frpc::Executor for Greeter {
    type State = ();
    fn execute<'fut, TR>(
        state: Self::State,
        id: u16,
        cursor: &'fut mut &[u8],
        transport: &'fut mut TR,
    ) -> Option<impl std::future::Future<Output = ()> + Send + 'fut>
    where
        TR: Transport + Send,
    {
        match id {
            1 => Output::_produce(SayHello, state, cursor, transport),
            2 => Output::_produce(SayHelloAgain, state, cursor, transport),
            _ => ::std::option::Option::None,
        }
    }
}
