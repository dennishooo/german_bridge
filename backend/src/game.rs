use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::connection::PlayerId;
use crate::game_state::GameState;

pub type GameId = Uuid;

pub struct GameManager {
    games: Arc<RwLock<HashMap<GameId, Game>>>,
}

pub struct Game {
    pub id: GameId,
    pub state: GameState,
    pub players: Vec<PlayerId>,
    pub created_at: Instant,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
