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

    /// Create a new game with the given players
    pub async fn create_game(&self, players: Vec<PlayerId>) -> GameId {
        let game_id = Uuid::new_v4();
        let game_state = GameState::new(players.clone());

        let game = Game {
            id: game_id,
            state: game_state,
            players,
            created_at: Instant::now(),
        };

        let mut games = self.games.write().await;
        games.insert(game_id, game);

        game_id
    }
}
