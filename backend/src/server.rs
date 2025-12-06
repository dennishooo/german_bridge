use crate::error::ServerError;
use crate::connection::{ConnectionManager, PlayerId};
use crate::protocol::{ClientMessage, ServerMessage};
use crate::game::GameManager;
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, State, Query},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};
use futures::{StreamExt, SinkExt};

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub turn_timeout_secs: u64,
    pub log_level: String,
}

pub struct AppState {
    pub connection_manager: Arc<ConnectionManager>,
    pub game_manager: Arc<GameManager>,
}

pub async fn run_server(config: ServerConfig) -> Result<(), ServerError> {
    let addr = format!("{}:{}", config.host, config.port);
    
    info!("Starting server on {}", addr);
    info!("Configuration: max_connections={}, turn_timeout={}s, log_level={}", 
          config.max_connections, config.turn_timeout_secs, config.log_level);
    
    // Create shared state
    let connection_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(Arc::clone(&connection_manager)));
    
    let app_state = Arc::new(AppState {
        connection_manager,
        game_manager,
    });
    
    // Build the Axum router with shared state
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_check))
        .route("/stats", get(stats_handler))
        .with_state(app_state);
    
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

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Check if this is a reconnection attempt
    let reconnect_player_id = params.get("player_id")
        .and_then(|id| id.parse::<PlayerId>().ok());
    
    let connection_manager = Arc::clone(&app_state.connection_manager);
    ws.on_upgrade(move |socket| handle_socket(socket, connection_manager, reconnect_player_id))
}

async fn handle_socket(
    socket: WebSocket,
    connection_manager: Arc<ConnectionManager>,
    reconnect_player_id: Option<PlayerId>,
) {
    info!("New WebSocket connection established");
    
    // Split the WebSocket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();
    
    // Create a channel for sending messages to this WebSocket
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    
    // Handle reconnection or new connection
    let (player_id, is_reconnection) = if let Some(reconnect_id) = reconnect_player_id {
        // Attempt to reconnect
        match connection_manager.reconnect_player(reconnect_id, tx.clone()).await {
            Some(other_players) => {
                info!("Player {} reconnected successfully", reconnect_id);
                
                // Send Connected message
                let connected_msg = ServerMessage::Connected { player_id: reconnect_id };
                if let Ok(json) = serde_json::to_string(&connected_msg) {
                    if let Err(e) = ws_sender.send(Message::Text(json)).await {
                        error!("Failed to send Connected message to player {}: {}", reconnect_id, e);
                        return;
                    }
                }
                
                // Notify other players about reconnection
                if !other_players.is_empty() {
                    connection_manager.broadcast_to_players(
                        &other_players,
                        ServerMessage::PlayerReconnected { player_id: reconnect_id }
                    ).await;
                }
                
                (reconnect_id, true)
            }
            None => {
                warn!("Reconnection failed for player {} (timeout or not found), creating new session", reconnect_id);
                // Reconnection failed, create new session
                let new_id = connection_manager.add_player(tx.clone()).await;
                
                // Send Connected message
                let connected_msg = ServerMessage::Connected { player_id: new_id };
                if let Ok(json) = serde_json::to_string(&connected_msg) {
                    if let Err(e) = ws_sender.send(Message::Text(json)).await {
                        error!("Failed to send Connected message to player {}: {}", new_id, e);
                        connection_manager.remove_player(new_id).await;
                        return;
                    }
                }
                
                (new_id, false)
            }
        }
    } else {
        // New connection
        let player_id = connection_manager.add_player(tx).await;
        
        // Send Connected message with player_id
        let connected_msg = ServerMessage::Connected { player_id };
        if let Ok(json) = serde_json::to_string(&connected_msg) {
            if let Err(e) = ws_sender.send(Message::Text(json)).await {
                error!("Failed to send Connected message to player {}: {}", player_id, e);
                connection_manager.remove_player(player_id).await;
                return;
            }
        }
        
        (player_id, false)
    };
    
    if is_reconnection {
        info!("Player {} reconnected and restored", player_id);
    } else {
        info!("Player {} connected and registered", player_id);
    }
    
    // Spawn a task to forward messages from the channel to the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });
    
    // Spawn a task to receive messages from the WebSocket
    let connection_manager_clone = connection_manager.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(msg) => {
                    if let Err(e) = handle_message(player_id, msg, &connection_manager_clone).await {
                        warn!("Error handling message from player {}: {}", player_id, e);
                    }
                }
                Err(e) => {
                    warn!("WebSocket error for player {}: {}", player_id, e);
                    break;
                }
            }
        }
        player_id
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = &mut send_task => {
            debug!("Send task completed for player {}", player_id);
            recv_task.abort();
        }
        result = &mut recv_task => {
            debug!("Receive task completed for player {}", player_id);
            send_task.abort();
            if let Ok(player_id) = result {
                // Mark player as inactive and get list of other players to notify
                let other_players = connection_manager.mark_inactive(player_id).await;
                
                // Notify other players about the disconnection
                if !other_players.is_empty() {
                    connection_manager.broadcast_to_players(
                        &other_players,
                        ServerMessage::PlayerLeft { player_id }
                    ).await;
                }
            }
        }
    }
    
    info!("Player {} disconnected", player_id);
}

async fn handle_message(
    player_id: crate::connection::PlayerId,
    msg: Message,
    connection_manager: &ConnectionManager,
) -> Result<(), String> {
    // Update player activity
    connection_manager.update_activity(player_id).await;
    
    match msg {
        Message::Text(text) => {
            debug!("Received text message from player {}: {}", player_id, text);
            
            // Deserialize the message
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(client_msg) => {
                    debug!("Parsed message from player {}: {:?}", player_id, client_msg);
                    // TODO: Route message to appropriate handler
                    // For now, just handle Ping
                    if matches!(client_msg, ClientMessage::Ping) {
                        connection_manager.send_to_player(player_id, ServerMessage::Pong).await;
                    }
                    Ok(())
                }
                Err(e) => {
                    let error_msg = format!("Invalid message format: {}", e);
                    warn!("Failed to parse message from player {}: {}", player_id, error_msg);
                    connection_manager.send_to_player(
                        player_id,
                        ServerMessage::Error { message: error_msg.clone() }
                    ).await;
                    Err(error_msg)
                }
            }
        }
        Message::Binary(data) => {
            debug!("Received binary message from player {} ({} bytes)", player_id, data.len());
            
            // Try to deserialize from binary JSON
            match serde_json::from_slice::<ClientMessage>(&data) {
                Ok(client_msg) => {
                    debug!("Parsed binary message from player {}: {:?}", player_id, client_msg);
                    // TODO: Route message to appropriate handler
                    // For now, just handle Ping
                    if matches!(client_msg, ClientMessage::Ping) {
                        connection_manager.send_to_player(player_id, ServerMessage::Pong).await;
                    }
                    Ok(())
                }
                Err(e) => {
                    let error_msg = format!("Invalid binary message format: {}", e);
                    warn!("Failed to parse binary message from player {}: {}", player_id, error_msg);
                    connection_manager.send_to_player(
                        player_id,
                        ServerMessage::Error { message: error_msg.clone() }
                    ).await;
                    Err(error_msg)
                }
            }
        }
        Message::Close(_) => {
            info!("Received close message from player {}", player_id);
            Err("Connection closed by client".to_string())
        }
        Message::Ping(_) | Message::Pong(_) => {
            // WebSocket ping/pong frames are handled automatically
            Ok(())
        }
    }
}

async fn health_check() -> impl IntoResponse {
    "OK"
}

async fn stats_handler(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let connection_stats = app_state.connection_manager.get_stats().await;
    let game_stats = app_state.game_manager.get_stats().await;
    
    let stats = ServerStats {
        connections: connection_stats,
        games: game_stats,
    };
    
    Json(stats)
}

#[derive(Debug, Clone, serde::Serialize)]
struct ServerStats {
    connections: crate::connection::ConnectionStats,
    games: crate::game::GameStats,
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
