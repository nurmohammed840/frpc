mod greeter;
mod server_sent_events;
mod stateful;

pub use greeter::Greeter;
pub use server_sent_events::ServerSentEvents;
pub use stateful::Stateful;

// 3rd perty library can simplify this process.
pub fn codegen_init() {
    #[cfg(debug_assertions)]
    {
        use frpc_codegen_client::{typescript, Config};
        Config {
            typescript: Some(typescript::Config {
                out_dir: "./examples/out/".into(),
                preserve_import_extension: true,
                ..Default::default()
            }),
            ..Default::default()
        }
        .generate_binding(&[&Greeter.into(), &ServerSentEvents.into(), &Stateful.into()])
        .expect("failed to generate bindings");
    }
}
