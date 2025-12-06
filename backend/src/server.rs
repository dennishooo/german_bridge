use crate::error::ServerError;

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
}

pub async fn run_server(config: ServerConfig) -> Result<(), ServerError> {
    // TODO: Implement server initialization
    Ok(())
}
