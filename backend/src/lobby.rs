use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::connection::PlayerId;

pub type LobbyId = Uuid;

pub struct LobbyManager {
    lobbies: Arc<RwLock<HashMap<LobbyId, Lobby>>>,
}

pub struct Lobby {
    pub id: LobbyId,
    pub host: PlayerId,
    pub players: Vec<PlayerId>,
    pub max_players: usize,
    pub created_at: Instant,
}

impl LobbyManager {
    pub fn new() -> Self {
        Self {
            lobbies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
