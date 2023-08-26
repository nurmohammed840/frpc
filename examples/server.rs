mod src;

use frpc_transport_http::{http, Config, Ctx, Server};
use src::*;
use std::{fs, io};

static TRANSPORT_CONFIG: Config = Config::new();

#[tokio::main]
async fn main() -> io::Result<()> {
    codegen_init();

    let key = fs::read("./examples/key.pem")?;
    let certs = fs::read("./examples/cert.pem")?;
    let server = Server::bind("127.0.0.1:4433", &mut &*certs, &mut &*key)
        .await
        .unwrap();

    println!("Server Runing at {}\n", server.local_addr()?);

    loop {
        if let Ok((conn, addr)) = server.accept().await {
            conn.incoming(
                (addr, Default::default()),
                |_, (addr, user), mut req, mut res| async move {
                    println!("From: {addr}; {req:#?}");
                    let mut ctx = Ctx::new(&mut req, &mut res);

                    res.status = match ctx.req.uri.path() {
                        "/greeter" => ctx.serve(&TRANSPORT_CONFIG, (), Greeter::execute).await,
                        "/stateful" => ctx.serve(&TRANSPORT_CONFIG, user, Stateful::execute).await,
                        "/sse" => {
                            ctx.serve(&TRANSPORT_CONFIG, (), ServerSentEvents::execute)
                                .await
                        }
                        _ => http::StatusCode::NOT_FOUND,
                    };
                    if res.status != http::StatusCode::OK {
                        let _ = res.write("Bad Request!").await;
                    }
                },
                |(addr, _)| async move { println!("Connection Closed: {addr}") },
            )
        }
    }
}
