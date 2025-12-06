use crate::error::ServerError;
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::signal;
use tracing::info;

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub turn_timeout_secs: u64,
    pub log_level: String,
}

pub async fn run_server(config: ServerConfig) -> Result<(), ServerError> {
    let addr = format!("{}:{}", config.host, config.port);
    
    info!("Starting server on {}", addr);
    info!("Configuration: max_connections={}, turn_timeout={}s, log_level={}", 
          config.max_connections, config.turn_timeout_secs, config.log_level);
    
    // Build the Axum router
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_check));
    
    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| ServerError::Io(e))?;
    
    info!("Server listening on {}", addr);
    
    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| ServerError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    info!("Server shutdown complete");
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    // TODO: Implement WebSocket connection handling
    info!("New WebSocket connection established");
    
    // For now, just close the connection
    let _ = socket.close().await;
}

async fn health_check() -> impl IntoResponse {
    "OK"
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal, shutting down gracefully");
        },
        _ = terminate => {
            info!("Received terminate signal, shutting down gracefully");
        },
    }
}
