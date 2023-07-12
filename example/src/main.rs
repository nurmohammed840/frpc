mod greeter;
mod server_sent_events;
mod stateful;

use frpc_transport_http::{http, Config, Ctx, Server};
use std::{fs, io, ops::ControlFlow};

use greeter::Greeter;
use server_sent_events::ServerSentEvents;
use stateful::Stateful;

static TRANSPORT_CONFIG: Config = Config::new();

#[tokio::main]
async fn main() -> io::Result<()> {
    codegen_init();

    // HTTP Boilerplate

    let addr = "127.0.0.1:4433";
    println!("Server Runing at {addr}\n");

    Server::bind(
        addr,
        &mut fs::read("./cert.pem")?.as_ref(),
        &mut fs::read("./key.pem")?.as_ref(),
    )
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

// 3rd perty library can simplify this process.
fn codegen_init() {
    #[cfg(debug_assertions)]
    {
        use frpc_codegen_client::{typescript, Config};
        Config {
            typescript: Some(typescript::Config {
                out_dir: "./out".into(),
                preserve_import_extension: true,
                ..Default::default()
            }),
            ..Default::default()
        }
        .generate_binding(&[&Greeter.into(), &ServerSentEvents.into(), &Stateful.into()])
        .expect("failed to generate bindings");
    }
}
