mod src;

use frpc_transport_http::{http, Config};
use rpc::*;
use src::*;
use std::{io, net::SocketAddr, sync::Arc};

static RPC_CONFIG: Config = Config::new();

#[tokio::main]
async fn main() -> io::Result<()> {
    codegen_init();
    println!("Server Runing at 127.0.0.1:4433");

    let server = Server::new("./examples/key.pem", "./examples/cert.pem")?;
    server
        .bind("127.0.0.1:4433", |addr, _| async move {
            App {
                addr,
                user: Default::default(),
            }
        })
        .await
}

#[derive(Clone)]
struct App {
    addr: SocketAddr,
    user: Arc<src::User>,
}

impl Application for App {
    async fn stream(self, mut ctx: Ctx) {
        println!("From: {}; {:#?}", self.addr, ctx.req);

        ctx.res.status = match ctx.req.uri.path() {
            "/greeter" => ctx.serve(&RPC_CONFIG, (), Greeter::execute).await,
            "/stateful" => ctx.serve(&RPC_CONFIG, self.user, Stateful::execute).await,
            "/sse" => ctx.serve(&RPC_CONFIG, (), ServerSentEvents::execute).await,
            _ => http::StatusCode::NOT_FOUND,
        };
        if ctx.res.status != http::StatusCode::OK {
            let _ = ctx.res.write("Bad Request!").await;
        }
    }
    async fn close(self) {
        println!("Connection Closed: {}", self.addr);
    }
}
