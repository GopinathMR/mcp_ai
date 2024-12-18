use std::{
    collections::HashMap,
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use bytes::Bytes;
use http_body_util::Full;
use hyper::{server::conn::http1, service::service_fn};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use jsonrpsee::{server::Server, types::ErrorObjectOwned};
use protocol::{ClientInfo, InitializeResponse};
use tokio::net::TcpListener;

use axum::{
    async_trait,
    http::method,
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use futures::stream::{self, Stream};
use jsonrpsee::proc_macros::rpc;
use tower_layer::Identity;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt as _;
use tower_http::trace::TraceLayer;

use crate::server::protocol::McpProtocolServer;

mod protocol;

#[derive(Clone, Copy)]
pub struct McpServer {
    sse_port: u16,
    http_port: u16,
}

impl McpServer {
    pub fn new(sse_port: u16, http_port: u16) -> Self {
        Self {
            sse_port,
            http_port,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        let sse_server = self.build_sse_server();
        let http_server = self.build_http_server();
        let _ret = futures_util::join!(sse_server, http_server);
        Ok(())
    }

    async fn build_sse_server(&self) -> Result<(), Box<dyn Error>> {
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), self.sse_port);
        println!("Starting SSE server listening on {}", self.sse_port);
        let listener = TcpListener::bind(socket).await?;
        tracing::debug!(
            "SSE server startedlistening on {}",
            listener.local_addr().unwrap()
        );
        println!(
            "SSE server started listening on {}",
            listener.local_addr().unwrap()
        );
        axum::serve(listener, self.build_sse_router()).await?;
        Ok(())
    }

    fn build_sse_router(&self) -> Router {
        Router::new()
            .route("/sse", get(sse_handler))
            .layer(TraceLayer::new_for_http())
    }

    async fn build_jsonrpc_server(&self) -> Result<Server<Identity , Identity>, Box<dyn Error>> {
        let server = Server::builder().build("127.0.0.1:8081").await?;
        Ok(server)
    }

    async fn build_http_server(&self) -> Result<(), Box<dyn Error>> {
        println!("Starting HTTP server listening on {}", self.http_port);
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), self.http_port);
        let listener = TcpListener::bind(socket).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(http_handler))
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

async fn http_handler(
    _: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    Ok(Response::new(Full::new(Bytes::from("Hello World"))))
}

async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    let stream = stream::repeat_with(|| {
        Event::default()
            .event("endpoint")
            .data(urlencoding::encode("http://localhost:8081"))
    })
    .map(Ok)
    .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

#[allow(non_snake_case)]
#[async_trait]
impl McpProtocolServer for McpServer {
    async fn initialize(
        &self,
        protocolVersion: &str,
        capabilities: HashMap<String, String>,
        clientInfo: ClientInfo,
    ) -> Result<InitializeResponse, ErrorObjectOwned> {
        Ok(InitializeResponse::default())
    }

    async fn initialized(&self) -> Result<(), ErrorObjectOwned> {
        Ok(())
    }
}
