use crate::server::ServerConfig;
use std::env;

pub fn load_config() -> ServerConfig {
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    
    let port = env::var("SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    
    let max_connections = env::var("MAX_CONNECTIONS")
        .ok()
        .and_then(|m| m.parse().ok())
        .unwrap_or(1000);
    
    let turn_timeout_secs = env::var("TURN_TIMEOUT_SECS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(30);
    
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    
    ServerConfig {
        host,
        port,
        max_connections,
        turn_timeout_secs,
        log_level,
    }
}
