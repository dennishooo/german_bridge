use crate::server::ServerConfig;

pub fn load_config() -> ServerConfig {
    // TODO: Implement configuration loading
    ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        max_connections: 1000,
    }
}
