mod src;

use frpc_transport_http::{http, Config, Ctx, Server};
use src::*;
use std::{fs, io, ops::ControlFlow};

static TRANSPORT_CONFIG: Config = Config::new();

#[tokio::main]
async fn main() -> io::Result<()> {
    let key = fs::read("./examples/key.pem")?;
    let certs = fs::read("./examples/cert.pem")?;

    codegen_init();

    let addr = "127.0.0.1:4433";
    println!("Server Runing at {addr}\n");

    Server::bind(addr, &mut certs.as_ref(), &mut key.as_ref())
        .await
        .expect("failed to run http/2 server")
        .serve(
            |addr| async move {
                println!("New Connection: {addr}");
                ControlFlow::Continue(Some((addr, Default::default())))
            },
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
        .await;
    Ok(())
}
