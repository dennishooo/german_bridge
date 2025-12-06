use german_bridge_backend::{server, config, connection, game, lobby, router};
use std::sync::Arc;
use std::panic;

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

    // Set up panic handler to prevent server crashes
    panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic payload"
        };
        
        let location = if let Some(location) = panic_info.location() {
            format!("{}:{}:{}", location.file(), location.line(), location.column())
        } else {
            "Unknown location".to_string()
        };
        
        tracing::error!("PANIC occurred at {}: {}", location, message);
        tracing::error!("Panic backtrace: {:?}", std::backtrace::Backtrace::capture());
    }));

    tracing::info!("German Bridge Backend starting...");

    // Initialize Database
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:users.db".to_string());
    tracing::info!("Connecting to database at {}", database_url);

    // Create database file if it doesn't exist (sqlite only)
    if !std::path::Path::new("users.db").exists() {
        std::fs::File::create("users.db").expect("Failed to create database file");
    }

    let db_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");
    
    tracing::info!("Database migrations applied");
    
    // Initialize ConnectionManager with Arc
    let connection_manager = Arc::new(connection::ConnectionManager::new());
    tracing::info!("ConnectionManager initialized");
    
    // Initialize GameManager with ConnectionManager reference
    let game_manager = Arc::new(game::GameManager::new(Arc::clone(&connection_manager)));
    tracing::info!("GameManager initialized");
    
    // Initialize LobbyManager with GameManager and ConnectionManager references
    let lobby_manager = Arc::new(lobby::LobbyManager::new(Arc::clone(&game_manager), Arc::clone(&connection_manager)));
    tracing::info!("LobbyManager initialized");
    
    // Create MessageRouter with all manager references
    let message_router = Arc::new(router::MessageRouter::new(
        Arc::clone(&lobby_manager),
        Arc::clone(&game_manager),
        Arc::clone(&connection_manager),
    ));
    tracing::info!("MessageRouter initialized");
    
    // Start the server
    if let Err(e) = server::run_server(config, connection_manager, game_manager, message_router, db_pool).await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
