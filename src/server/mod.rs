use std::{error::Error, net::{IpAddr, Ipv4Addr, SocketAddr}};

use bytes::Bytes;
use http_body_util::Full;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::{rt::TokioIo};
use tokio::net::TcpListener;
use hyper::{Request, Response};

use axum::{
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use futures::stream::{self, Stream};
use std::{convert::Infallible, path::PathBuf, time::Duration};
use tokio_stream::StreamExt as _;
use tower_http::trace::TraceLayer;


pub struct McpServer {
    sse_port: u16,
    http_port: u16,
}

impl McpServer {
    pub fn new(sse_port: u16, http_port: u16) -> Self {
        Self { sse_port, http_port }
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        let sse_server = self.build_sse_server2();
        let http_server = self.build_http_server();    
        let _ret = futures_util::join!(sse_server, http_server);
        Ok(())
    }

    async fn build_sse_server2(&self) -> Result<(), Box<dyn Error>> {
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), self.sse_port);
        println!("Starting SSE server listening on {}", self.sse_port);
        let listener = TcpListener::bind(socket).await?;
        tracing::debug!("SSE server startedlistening on {}", listener.local_addr().unwrap());
        println!("SSE server started listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, self.build_sse_router()).await?;
        Ok(())
    }

    fn build_sse_router(&self) -> Router {
        Router::new().route("/sse", get(sse_handler2)).layer(TraceLayer::new_for_http())
    }



    async fn _build_sse_server(&self) -> Result<(), Box<dyn Error>> {
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), self.sse_port);
        let listener = TcpListener::bind(socket).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
    
                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service_fn(sse_handler))
                        .await
                    {
                        println!("Error serving connection: {:?}", err);
                    }
                });
        }
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

async fn sse_handler( _: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    Ok(Response::new(Full::new(Bytes::from("http://127.0.0.1:8081"))))
}

async fn http_handler( _: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    Ok(Response::new(Full::new(Bytes::from("Hello World"))))
}


async fn sse_handler2(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    // A `Stream` that repeats an event every second
    //
    // You can also create streams from tokio channels using the wrappers in
    // https://docs.rs/tokio-stream
    let stream = stream::repeat_with(|| Event::default().data("http://127.0.0.1:8081!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}    
