use german_bridge_backend::{server, config};

#[tokio::main]
async fn main() {
    // Load configuration first to get log level
    let config = config::load_config();
    
    // Initialize tracing with configured log level
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level))
        )
        .init();

    tracing::info!("German Bridge Backend starting...");
    
    // Start the server
    if let Err(e) = server::run_server(config).await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
