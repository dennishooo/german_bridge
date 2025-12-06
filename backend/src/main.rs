mod server;
mod config;
mod connection;
mod lobby;
mod game;
mod game_state;
mod protocol;
mod router;
mod error;
mod game_logic;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("German Bridge Backend starting...");
    
    // TODO: Load configuration and start server
}
