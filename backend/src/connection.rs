use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use axum::extract::ws::Message;
use crate::protocol::ServerMessage;
use tracing::{debug, warn, info};

pub type PlayerId = String;

const DEFAULT_RECONNECT_TIMEOUT_SECS: u64 = 60;

pub struct ConnectionManager {
    sessions: Arc<RwLock<HashMap<PlayerId, PlayerSession>>>,
    reconnect_timeout: Duration,
}

pub struct PlayerSession {
    pub id: PlayerId,
    pub ws_sender: mpsc::UnboundedSender<Message>,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub is_active: bool,
    pub disconnected_at: Option<Instant>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::with_reconnect_timeout(Duration::from_secs(DEFAULT_RECONNECT_TIMEOUT_SECS))
    }

    pub fn with_reconnect_timeout(reconnect_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            reconnect_timeout,
        }
    }

    /// Register a new player connection with a random ID and return it
    pub async fn add_player(&self, ws_sender: mpsc::UnboundedSender<Message>) -> PlayerId {
        let player_id = Uuid::new_v4().to_string();
        self.register_player(player_id.clone(), ws_sender).await;
        player_id
    }

    /// Register a player with a specific ID (used for auth)
    pub async fn register_player(&self, player_id: PlayerId, ws_sender: mpsc::UnboundedSender<Message>) {
        let now = Instant::now();
        
        let session = PlayerSession {
            id: player_id.clone(),
            ws_sender,
            connected_at: now,
            last_activity: now,
            is_active: true,
            disconnected_at: None,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(player_id.clone(), session);
        
        debug!("Player {} connected", player_id);
    }

    /// Remove a player connection
    pub async fn remove_player(&self, player_id: PlayerId) {
        let mut sessions = self.sessions.write().await;
        if sessions.remove(&player_id).is_some() {
            debug!("Player {} removed", player_id);
        }
    }

    /// Send a message to a specific player
    pub async fn send_to_player(&self, player_id: PlayerId, msg: ServerMessage) {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(&player_id) {
            if session.is_active {
                let json = match serde_json::to_string(&msg) {
                    Ok(json) => json,
                    Err(e) => {
                        warn!("Failed to serialize message for player {}: {}", player_id, e);
                        return;
                    }
                };
                
                if let Err(e) = session.ws_sender.send(Message::Text(json)) {
                    warn!("Failed to send message to player {}: {}", player_id, e);
                }
            }
        } else {
            warn!("Attempted to send message to non-existent player {}", player_id);
        }
    }

    /// Broadcast a message to multiple players
    pub async fn broadcast_to_players(&self, player_ids: &[PlayerId], msg: ServerMessage) {
        let json = match serde_json::to_string(&msg) {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to serialize broadcast message: {}", e);
                return;
            }
        };
        
        let sessions = self.sessions.read().await;
        
        for player_id in player_ids {
            if let Some(session) = sessions.get(player_id) {
                if session.is_active {
                    if let Err(e) = session.ws_sender.send(Message::Text(json.clone())) {
                        warn!("Failed to broadcast to player {}: {}", player_id, e);
                    }
                }
            }
        }
    }

    /// Mark a player as inactive (disconnected)
    pub async fn mark_inactive(&self, player_id: PlayerId) -> Vec<PlayerId> {
        let mut sessions = self.sessions.write().await;
        let mut other_players = Vec::new();
        
        if let Some(session) = sessions.get_mut(&player_id) {
            session.is_active = false;
            session.disconnected_at = Some(Instant::now());
            info!("Player {} marked as inactive", player_id);
            
            // Collect all other active players to notify
            for (id, s) in sessions.iter() {
                if *id != player_id && s.is_active {
                    other_players.push(id.clone());
                }
            }
        }
        
        other_players
    }

    /// Reconnect a player with a new WebSocket sender
    pub async fn reconnect_player(&self, player_id: PlayerId, ws_sender: mpsc::UnboundedSender<Message>) -> Option<Vec<PlayerId>> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&player_id) {
            // Check if reconnection timeout has expired
            if let Some(disconnected_at) = session.disconnected_at {
                if disconnected_at.elapsed() > self.reconnect_timeout {
                    info!("Player {} reconnection timeout expired", player_id);
                    return None;
                }
            }
            
            session.ws_sender = ws_sender;
            session.is_active = true;
            session.last_activity = Instant::now();
            session.disconnected_at = None;
            info!("Player {} reconnected", player_id);
            
            // Collect all other active players to notify
            let mut other_players = Vec::new();
            for (id, s) in sessions.iter() {
                if *id != player_id && s.is_active {
                    other_players.push(id.clone());
                }
            }
            
            Some(other_players)
        } else {
            None
        }
    }

    /// Update last activity timestamp for a player
    pub async fn update_activity(&self, player_id: PlayerId) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&player_id) {
            session.last_activity = Instant::now();
        }
    }

    /// Check for expired inactive sessions and remove them
    pub async fn cleanup_expired_sessions(&self) -> Vec<PlayerId> {
        let mut sessions = self.sessions.write().await;
        let mut expired_players = Vec::new();
        
        let now = Instant::now();
        sessions.retain(|player_id, session| {
            if !session.is_active {
                if let Some(disconnected_at) = session.disconnected_at {
                    if now.duration_since(disconnected_at) > self.reconnect_timeout {
                        info!("Removing expired session for player {}", player_id);
                        expired_players.push(player_id.clone());
                        return false;
                    }
                }
            }
            true
        });
        
        expired_players
    }

    /// Get all active player IDs
    pub async fn get_active_players(&self) -> Vec<PlayerId> {
        let sessions = self.sessions.read().await;
        sessions.iter()
            .filter(|(_, session)| session.is_active)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get connection statistics
    pub async fn get_stats(&self) -> ConnectionStats {
        let sessions = self.sessions.read().await;
        let total_connections = sessions.len();
        let active_connections = sessions.iter()
            .filter(|(_, session)| session.is_active)
            .count();
        let inactive_connections = total_connections - active_connections;

        ConnectionStats {
            total_connections,
            active_connections,
            inactive_connections,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub inactive_connections: usize,
}
