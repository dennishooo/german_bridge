use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type PlayerId = Uuid;

pub struct ConnectionManager {
    sessions: Arc<RwLock<HashMap<PlayerId, PlayerSession>>>,
}

pub struct PlayerSession {
    pub id: PlayerId,
    pub connected_at: Instant,
    pub last_activity: Instant,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
