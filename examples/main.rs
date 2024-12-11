use mcp_ai::server::McpServer;
use tracing::{dispatcher::with_default, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, prelude::*};


#[tokio::main]
async fn main() {
    tracing_subscriber::registry().with(fmt::layer()).with(EnvFilter::from_default_env()).init();
    info!("Starting MCP Server");
    let server = McpServer::new(8080, 8081);
    let _res = server.start().await.unwrap();
}