pub use frpc_transport_http::Ctx;
use frpc_transport_http::{
    self as http,
    tokio_tls_listener::{rustls, tokio_rustls::server::TlsStream},
    Conn,
};
use std::{future::Future, io, net::SocketAddr, path::Path, sync::Arc};
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct Server {
    pub config: Arc<rustls::ServerConfig>,
}

impl Server {
    #[inline]
    pub fn new(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            config: Arc::new(http::Server::config(key, cert)?),
        })
    }

    pub async fn bind<Fut, App>(
        &self,
        addr: impl ToSocketAddrs,
        mut app: impl FnMut(SocketAddr, &mut Conn<TlsStream<TcpStream>>) -> Fut,
    ) -> io::Result<()>
    where
        Fut: Future<Output = App>,
        App: Application,
    {
        let server = http::Server::bind(addr, self.config.clone()).await?;
        loop {
            if let Ok((mut conn, addr)) = server.accept().await {
                let app = app(addr, &mut conn).await;
                tokio::spawn(async move {
                    while let Some(Ok((req, res))) = conn.accept().await {
                        let app = app.clone();
                        tokio::spawn(async move { app.stream(Ctx::new(req, res)).await });
                    }
                    app.close().await;
                });
            }
        }
    }
}

pub trait Application: Clone + Send + 'static {
    fn stream(self, ctx: Ctx) -> impl Future<Output = ()> + Send;
    fn close(self) -> impl Future<Output = ()> + Send {
        async {}
    }
}

#[macro_export]
macro_rules! serve {
    ($ctx: ident: $($path: pat => $service: ident; $state: expr)*) => ({
        if let Some(status) = match $ctx.req.uri.path() {
            $($path => Some($ctx.serve($service, $state).await)),*,
            _ => None
        } {
            $ctx.res.status = status;
        }
    });
}
