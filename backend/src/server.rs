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
    pub message_router: Arc<crate::router::MessageRouter>,
    pub db: sqlx::SqlitePool,
}

pub async fn run_server(
    config: ServerConfig,
    connection_manager: Arc<ConnectionManager>,
    game_manager: Arc<GameManager>,
    message_router: Arc<crate::router::MessageRouter>,
    db_pool: sqlx::SqlitePool,
) -> Result<(), ServerError> {
    let addr = format!("{}:{}", config.host, config.port);
    
    info!("Starting server on {}", addr);
    info!("Configuration: max_connections={}, turn_timeout={}s, log_level={}", 
          config.max_connections, config.turn_timeout_secs, config.log_level);
    
    let app_state = Arc::new(AppState {
        connection_manager,
        game_manager,
        message_router,
        db: db_pool,
    });
    
    // CORS configuration
    let cors = tower_http::cors::CorsLayer::new()
        // Allow requests from any origin or specifically the frontend dev server
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::HeaderName::from_static("content-type"),
            axum::http::HeaderName::from_static("authorization"),
        ]);

    // Build the Axum router with shared state
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_check))
        .route("/stats", get(stats_handler))
        .route("/api/register", axum::routing::post(crate::handlers::auth::register))
        .route("/api/login", axum::routing::post(crate::handlers::auth::login))
        .layer(cors)
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
    // 1. JWT Authentication
    let token = params.get("token").cloned();
    let _reconnect_id = params.get("player_id").and_then(|id| id.parse::<PlayerId>().ok());
    
    let user_info = if let Some(token) = token {
        match crate::auth::verify_jwt(&token) {
            Ok(claims) => Some(claims),
            Err(e) => {
                warn!("Invalid JWT token: {}", e);
                // Return 401 if token invalid? WS handshake usually returns 400/401
                // But for now we might fail gracefully or allow anon if we wanted (but plan says protect)
                // Let's degrade to error log and maybe close connection later if we want strict enforcement
                // Ideally we reject the handshake here.
                return (axum::http::StatusCode::UNAUTHORIZED, "Invalid Token").into_response();
            }
        }
    } else {
        // No token provided. Strict auth requires token.
        warn!("No token provided for WebSocket connection");
        return (axum::http::StatusCode::UNAUTHORIZED, "Missing Token").into_response();
    };
    
    let user_id = user_info.unwrap().sub; // We know it's Some here because of return above

    // Pass validated user_id to handle_socket (we might want to replace the random PlayerId with this User ID)
    // Or we map UserID -> PlayerID in a new manager.
    // OPTION: We use the UserID AS the PlayerID. UUID string vs u32/string. Protocol uses String alias.
    // Let's use the User ID as the Player ID.
    
    ws.on_upgrade(move |socket| handle_socket(socket, app_state, user_id))
}

async fn handle_socket(
    socket: WebSocket,
    app_state: Arc<AppState>,
    authenticated_user_id: String,
) {
    let connection_manager = Arc::clone(&app_state.connection_manager);
    let message_router = Arc::clone(&app_state.message_router);
    info!("New Authenticated WebSocket connection: {}", authenticated_user_id);
    
    // Split the WebSocket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();
    
    // Create a channel for sending messages to this WebSocket
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    
    // FOR AUTH: We trust the JWT user_id.
    // Check if this user is already connected (reconnection) or new.
    // The connection_manager uses PlayerId (String). 
    // We can try to reconnect if they exist, or add if they don't.
    // BUT ConnectionManager currently generates random IDs for new players.
    // We need to modify/overload add_player to accept a specific ID, OR just use the AUTH ID.
    // Let's assume we want to use the AUTH ID as the Player ID.
    // This requires ConnectionManager to support "add_player_with_id".
    // Since we don't have that yet, I'll modify ConnectionManager or work around it.
    // WORKAROUND: For now, I'll use the authenticated_user_id.
    // I need to change how `connection_manager` works slightly or just try `reconnect_player`.
    // If `reconnect_player` fails (not connected), we need `add_player_with_id`.
    
    // Since I can't easily change ConnectionManager right now without reading it,
    // I'll stick to the existing `add_player` which generates a random ID, 
    // BUT this ignores the persisted User ID which defines identity.
    // CRITICAL: We MUST use the User ID as the Player ID for persistence to work properly across reloads.
    
    // I will try to use `reconnect_player` first. If it fails, I really should add them with their specific ID.
    // If ConnectionManager doesn't support custom IDs, I should add that capability.
    // For this step, I will assume I can just add them.
    // However, looking at previous code, `add_player` returns a new ID.
    
    // Let's modify this tool call to ONLY do the signature change, and then I'll inspect ConnectionManager.
    // Use a placeholder logic that attempts to use the ID.
    
    let player_id = authenticated_user_id.clone();
    
    // We try to reconnect first
    let is_reconnection = if let Some(other_players) = connection_manager.reconnect_player(player_id.clone(), tx.clone()).await {
        info!("Player {} (User) reconnected", player_id);
        
        // Send Connected message
        let connected_msg = ServerMessage::Connected { player_id: player_id.clone() };
        if let Ok(json) = serde_json::to_string(&connected_msg) {
            if let Err(e) = ws_sender.send(Message::Text(json)).await {
                error!("Failed to send Connected message: {}", e);
                return;
            }
        }
        
        // Broadcast
         if !other_players.is_empty() {
            connection_manager.broadcast_to_players(
                &other_players,
                ServerMessage::PlayerReconnected { player_id: player_id.clone() }
            ).await;
        }
        true
    } else {
        info!("User {} connecting as new session", player_id);
        
        // Register the authenticated user as a player
        connection_manager.register_player(player_id.clone(), tx).await;
        
        // Send Connected message with player_id
        let connected_msg = ServerMessage::Connected { player_id: player_id.clone() };
        if let Ok(json) = serde_json::to_string(&connected_msg) {
            if let Err(e) = ws_sender.send(Message::Text(json)).await {
                error!("Failed to send Connected message to player {}: {}", player_id, e);
                connection_manager.remove_player(player_id).await;
                return;
            }
        }
        
        false
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
    // Errors in this task are isolated and won't affect other connections
    let connection_manager_clone = connection_manager.clone();
    let message_router_clone = message_router.clone();
    let player_id_clone = player_id.clone();
    
    let mut recv_task = tokio::spawn(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(msg) => {
                    // Wrap message handling to catch any errors
                    if let Err(e) = handle_message(player_id_clone.clone(), msg, &connection_manager_clone, &message_router_clone).await {
                        warn!("Error handling message from player {}: {}", player_id_clone, e);
                        // Continue processing other messages despite error
                    }
                }
                Err(e) => {
                    warn!("WebSocket error for player {}: {}", player_id_clone, e);
                    break;
                }
            }
        }
        player_id_clone
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
                let other_players = connection_manager.mark_inactive(player_id.clone()).await;
                
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
    message_router: &crate::router::MessageRouter,
) -> Result<(), String> {
    // Update player activity
    connection_manager.update_activity(player_id.clone()).await;
    
    match msg {
        Message::Text(text) => {
            debug!("Received text message from player {}: {}", player_id, text);
            
            // Deserialize the message
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(client_msg) => {
                    debug!("Parsed message from player {}: {:?}", player_id, client_msg);
                    
                    // Route message to appropriate handler
                    if let Err(e) = message_router.route_message(player_id.clone(), client_msg).await {
                        let error_msg = format!("Failed to route message: {}", e);
                        warn!("Error routing message from player {}: {}", player_id, error_msg);
                        return Err(error_msg);
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
                    
                    // Route message to appropriate handler
                    if let Err(e) = message_router.route_message(player_id.clone(), client_msg).await {
                        let error_msg = format!("Failed to route message: {}", e);
                        warn!("Error routing message from player {}: {}", player_id, error_msg);
                        return Err(error_msg);
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
