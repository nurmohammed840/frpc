mod src;

use frpc_transport_http::{http, Config, Ctx, Incoming, Request, Response, Server};
use src::*;
use std::{io, net::SocketAddr, sync::Arc};

static TRANSPORT_CONFIG: Config = Config::new();

#[tokio::main]
async fn main() -> io::Result<()> {
    codegen_init();

    let config = Server::config("./examples/key.pem", "./examples/cert.pem")?;
    let server = Server::bind("127.0.0.1:4433", config).await?;

    println!("Server Runing at {}\n", server.local_addr()?);
    loop {
        if let Ok((conn, addr)) = server.accept().await {
            conn.incoming(App {
                addr,
                user: Default::default(),
            });
        }
    }
}

#[derive(Clone)]
struct App {
    addr: SocketAddr,
    user: Arc<src::User>,
}

impl Incoming for App {
    async fn stream(self, mut req: Request, mut res: Response) {
        println!("From: {}; {req:#?}", self.addr);
        let mut ctx = Ctx::new(&mut req, &mut res);

        res.status = match ctx.req.uri.path() {
            "/greeter" => ctx.serve(&TRANSPORT_CONFIG, (), Greeter::execute).await,
            "/stateful" => {
                ctx.serve(&TRANSPORT_CONFIG, self.user, Stateful::execute)
                    .await
            }
            "/sse" => {
                ctx.serve(&TRANSPORT_CONFIG, (), ServerSentEvents::execute)
                    .await
            }
            _ => http::StatusCode::NOT_FOUND,
        };
        if res.status != http::StatusCode::OK {
            let _ = res.write("Bad Request!").await;
        }
    }
    async fn close(self) {
        println!("Connection Closed: {}", self.addr);
    }
}
