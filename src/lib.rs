pub mod server;

pub fn add(left: u64, right: u64) -> u64 {
    
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        server::McpServer::new(8080, 8081).start();
        assert_eq!(result, 4);
    }
}
