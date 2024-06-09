pub use frpc::*;

use frpc_transport_http::{
    self as http,
    tokio_tls_listener::{rustls, tokio_rustls::server::TlsStream},
    Conn, Request, Response,
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

    pub async fn bind<Fut>(
        &self,
        addr: impl ToSocketAddrs,
        mut app: impl FnMut(SocketAddr, &mut Conn<TlsStream<TcpStream>>) -> Fut,
    ) -> io::Result<()>
    where
        Fut: Future, /* + Send */
    {
        // impl Connection
        let server = http::Server::bind(addr, self.config.clone()).await?;
        loop {
            if let Ok((mut conn, addr)) = server.accept().await {
                let g = app(addr, &mut conn).await;
            }
        }

        // loop {
        //     if let Ok((mut conn, addr)) = server.accept().await {
        //         let c = app.accept(addr).await;
        //         tokio::spawn(async move {
        //             while let Some(Ok((req, res))) = conn.accept().await {
        //                 let mut _c = c.clone();
        //                 tokio::spawn(async move { _c.stream(req, res).await });
        //             }
        //             c.close().await;
        //         });
        //     }
        // }
    }
}

// pub trait Connection: Clone + Send + 'static {
//     fn stream(self, req: Request, res: Response) -> impl Future<Output = ()> + Send;
//     fn close(self) -> impl Future<Output = ()> + Send {
//         async {}
//     }
// }

// #[derive(Clone)]
// struct App {}

// impl Connection for App {
//     async fn stream(self, _req: Request, _res: Response) {
//         todo!()
//     }
// }

async fn test_name() {
    let s = Server::new("key", "cert").unwrap();

    s.bind("addr", |addr, _| async move {
        println!("addr: {:#?}", addr);
    })
    .await;
}
