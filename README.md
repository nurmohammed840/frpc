A high performance
[Remote Procedure Call (RPC)](https://en.wikipedia.org/wiki/Remote_procedure_call)
system.

### Usage

Add this to your `Cargo.toml` file.

```toml
[dependencies]
frpc = { git = "https://github.com/nurmohammed840/frpc" }
frpc-transport = { git = "https://github.com/nurmohammed840/frpc" }

# Required for codegen
frpc-codegen-client = { git = "https://github.com/nurmohammed840/frpc" }
```

### Example

let's re-implement famous gRPC
[greeter](https://grpc.io/docs/languages/python/quickstart/) example in rust!

```rust
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

async fn SayHelloAgain(req: HelloRequest) -> HelloReply {
    HelloReply {
        message: format!("Hello Again, {}", req.name),
    }
}

declare! {
    /// The greeting service definition.
    service Greeter {
        /// Sends a greeting
        rpc SayHello = 1;

        /// Sends another greeting
        rpc SayHelloAgain = 2;
    }
}
```

A service is like a namespace. `Greeter` service has two functions with an
unique `u16` id. The ID is used to identify which function to call.

Function parameters have to derived from `Input` and output with `Output` macro.

Client then call those function as follow:

```ts
let greeter = new Greeter(new HttpTransport("<URL>"));
console.log(await greeter.SayHello({ name: "Foo!" })); // { message: "Hello Foo!" }
console.log(await greeter.SayHelloAgain({ name: "Foo!" })); // { message: "Hello Again, Foo!" }
```

You get the typesafe client API for free! Still not impressed ?!

### Server Stream Example

In this
[example](https://github.com/nurmohammed840/frpc/blob/main/examples/src/server_sent_events.rs)
server send a stream of messages.

```rust
use frpc::*;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Output)]
struct Event {
    elapsed: u64,
}

fn get_events(count: u8) -> impl Output {
    sse! {
        if count > 10 {
            return Err(format!("count: {count} should be <= 10"));
        }
        let time = Instant::now();
        for _ in 0..count {
            sleep(Duration::from_secs(1)).await;
            yield Event { elapsed: time.elapsed().as_secs() }
        }
        Ok(())
    }
}

declare! {
    service ServerSentEvents {
        rpc get_events = 1;
    }
}
```

Here `sse!` macro create an async generator, `impl Output` is used omit return
type, whereas return type would be:
`SSE<impl AsyncGenerator<Yield = Event, Return = Result<(), String>>>`

The client then calls this function as follow:

```ts
let sse = new ServerSentEvents(new HttpTransport("<URL>"));
for await (const ev of sse.get_events(3)) {
  console.log(ev);
}
```

It's that easy!

See more
[examples](https://github.com/nurmohammed840/frpc/tree/main/examples/src)

### Motivation

The idea behind any RPC system is to communicate locally or remotely without
writing any remote interactions.

This is usually done by generation some glue code, also known as
[Stub](https://en.wikipedia.org/wiki/Stub_(distributed_computing)), which is
responsible for network communication, conversion types and serializing function
parameters. Stub usually generatied from
[Domain Specific Language (DSL)](https://en.wikipedia.org/wiki/Domain-specific_language),
for example [gRPC](https://grpc.io/) use
[Protocol Buffers](https://protobuf.dev/), which describe an interface.

This library doesn't use any
[Interface Description Language (IDL)](https://en.wikipedia.org/wiki/Interface_description_language),
and the interface is described from the Rust codebase using macros.
