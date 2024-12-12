#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;
use std::collections::HashMap;

/// Protocol version supported by this implementation   
pub const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Serialize, Deserialize, Clone)]
pub struct InitializeRequest {
    pub protocolVersion: String,
    pub capabilities: HashMap<String, String>,
    pub sessionId: Option<String>,
    pub authentication: Option<String>,
    pub clientInfo: ClientInfo,
}

impl Default for InitializeRequest {
    fn default() -> Self {
        Self {
            protocolVersion: PROTOCOL_VERSION.to_string(),
            capabilities: HashMap::new(),
            sessionId: None,
            authentication: None,
            clientInfo: ClientInfo::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InitializeResponse {
    pub protocolVersion: String,
    pub capabilities: HashMap<String, String>,
    pub sessionId: Option<String>,
    pub authentication: Option<String>,
    pub serverInfo: ServerInfo,
}

impl Default for InitializeResponse {
    fn default() -> Self {
        Self {
            protocolVersion: PROTOCOL_VERSION.to_string(),
            capabilities: HashMap::new(),
            sessionId: None,
            authentication: None,
            serverInfo: ServerInfo::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}

#[rpc(server, client)]
#[allow(non_snake_case)]
pub trait McpProtocol {
    /// Initialize the client and server connection as per Claude MCP protocol
    ///
    /// This method establishes the initial connection between client and server following the Model Context Protocol (MCP).
    /// It performs the following:
    /// 1. Validates client authentication and capabilities
    /// 2. Negotiates protocol version and supported features
    /// 3. Establishes secure communication channel
    /// 4. Returns initialization status and server capabilities
    ///
    /// The response includes:
    /// - Protocol version
    /// - Server capabilities (supported methods, extensions)
    /// - Session ID for subsequent requests
    /// - Authentication status
    ///
    /// # Errors
    /// - Returns error if authentication fails
    /// - Returns error if protocol version mismatch
    /// - Returns error if required capabilities not supported
    #[method(name = "initialize")]
    #[allow(unused_variables)]
    async fn initialize(
        &self,
        protocolVersion: &str,
        capabilities: HashMap<String, String>,
        clientInfo: ClientInfo,
    ) -> Result<InitializeResponse, ErrorObjectOwned>;

    #[method(name = "initialized")]
    async fn initialized(&self) -> Result<(), ErrorObjectOwned>;
}
