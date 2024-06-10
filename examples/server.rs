mod src;

use rpc::*;
use src::*;
use std::{io, net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> io::Result<()> {
    codegen_init();

    println!("Server Runing at 127.0.0.1:4433");
    Server::new("./examples/key.pem", "./examples/cert.pem")?
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
        serve! {ctx:
            "/greeter" => Greeter; ()
            "/stateful" => Stateful; self.user
            "/sse" => ServerSentEvents; ()
        }
    }
    async fn close(self) {
        println!("Connection Closed: {}", self.addr);
    }
}
