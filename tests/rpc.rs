//! cargo test --test rpc
//! cargo test --test rpc <serve | codegen>
mod cancellation;
mod echo;
mod sse;
mod validate;

use frpc_transport_http::{http::HeaderValue, Config, Ctx, Server};
use std::{
    collections::HashSet,
    fs,
    io::Result,
    process::{Command, Output, Stdio},
    time::Instant,
};
use tokio::task;

use cancellation::Cancellation;
use echo::EchoTest;
use sse::SSETest;
use validate::ValidateTest;

static CONF: Config = Config::new();

fn codegen() {
    use frpc_codegen_client::{typescript, Config};
    let time = Instant::now();

    Config {
        typescript: Some(typescript::Config {
            out_dir: "./target/rpc".into(),
            preserve_import_extension: true,
        }),
    }
    .generate_binding(&[
        &EchoTest.into(),
        &ValidateTest.into(),
        &SSETest.into(),
        &Cancellation.into(),
    ])
    .expect("Failed to generate binding");

    println!("Codegen finished in {:?}\n", time.elapsed());
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: HashSet<_> = std::env::args().skip(1).collect();

    codegen();

    if args.contains("codegen") {
        return Ok(());
    }

    let cert = fs::read("examples/cert.pem")?;
    let key = fs::read("examples/key.pem")?;
    let server = Server::bind("127.0.0.1:4433", &mut &*cert, &mut &*key)
        .await
        .unwrap()
        .with_graceful_shutdown();

    let serve = async {
        loop {
            if let Ok((conn, _addr)) = server.accept().await {
                conn.incoming(
                    Default::default(),
                    |_conn, state, mut req, mut res| async move {
                        res.headers
                            .append("access-control-allow-origin", HeaderValue::from_static("*"));

                        let mut ctx = Ctx::new(&mut req, &mut res);
                        let _ = match ctx.req.uri.path() {
                            "/rpc/validate" => ctx.serve(&CONF, (), ValidateTest::execute).await,
                            "/rpc/echo" => ctx.serve(&CONF, state, EchoTest::execute).await,
                            "/rpc/sse" => ctx.serve(&CONF, (), SSETest::execute).await,
                            "/rpc/cancellation" => {
                                ctx.serve(&CONF, (), Cancellation::execute).await
                            }
                            _ => return,
                        };
                    },
                    |_| async {},
                )
            }
        }
    };

    if args.contains("serve") {
        println!("Server runing...");
        println!("Goto: https://127.0.0.1:4433/");
        serve.await;
    } else {
        tokio::select! {
            output = task::spawn_blocking(run_clients) => output??,
            _ = serve => {},
        }
    }
    server.shutdown().await;
    Ok(())
}

fn run_clients() -> Result<()> {
    run_js("./tests/echo/mod.ts")?;
    run_js("./tests/validate/mod.ts")?;
    run_js("./tests/sse/mod.ts")?;
    run_js("./tests/cancellation/mod.ts")?;
    Ok(())
}

fn run_js(path: &str) -> Result<Output> {
    Command::new("deno")
        .args([
            "run",
            "--allow-net=localhost",
            "--unsafely-ignore-certificate-errors=localhost",
            "--check",
            path,
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
}
