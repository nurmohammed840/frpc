//! cargo test --test rpc
//! cargo test --test rpc <serve | codegen>
mod cancellation;
mod echo;
mod sse;
mod validate;

use frpc_transport_http::{http::HeaderValue, Ctx, Incoming, Request, Response, Server};
use std::{
    collections::HashSet,
    io::Result,
    process::{Command, Output, Stdio},
    sync::Arc,
    time::Instant,
};
use tokio::task;

use cancellation::Cancellation;
use echo::EchoTest;
use sse::SSETest;
use validate::ValidateTest;


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

    let config = Server::config("examples/key.pem", "examples/cert.pem")?;
    let server = Server::bind("127.0.0.1:4433", config)
        .await?
        .with_graceful_shutdown();

    let serve = async {
        loop {
            if let Ok((conn, _addr)) = server.accept().await {
                conn.incoming(App {
                    state: Default::default(),
                });
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

#[derive(Clone)]
struct App {
    state: Arc<echo::Context>,
}

impl Incoming for App {
    async fn stream(self, req: Request, mut res: Response) {
        res.headers
            .append("access-control-allow-origin", HeaderValue::from_static("*"));

        let mut ctx = Ctx::new(req, res);

        let _ = match ctx.req.uri.path() {
            "/rpc/validate" => ctx.serve(ValidateTest, ()).await,
            "/rpc/echo" => ctx.serve(EchoTest, self.state).await,
            "/rpc/sse" => ctx.serve(SSETest, ()).await,
            "/rpc/cancellation" => ctx.serve(Cancellation, ()).await,
            _ => return,
        };
    }
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
