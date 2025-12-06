use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;
use crate::connection::{PlayerId, ConnectionManager};
use crate::game_state::GameState;
use crate::protocol::{ServerMessage, PlayerAction, PlayerGameView};
use crate::error::GameError;
use tracing::{debug, info, warn};

pub type GameId = Uuid;

pub struct GameManager {
    games: Arc<RwLock<HashMap<GameId, Game>>>,
    connection_manager: Arc<ConnectionManager>,
    timer_handles: Arc<RwLock<HashMap<GameId, JoinHandle<()>>>>,
}

pub struct Game {
    pub id: GameId,
    pub state: GameState,
    pub players: Vec<PlayerId>,
    pub created_at: Instant,
}

impl GameManager {
    /// Create a new GameManager with a reference to ConnectionManager
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            games: Arc::new(RwLock::new(HashMap::new())),
            connection_manager,
            timer_handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Helper method to get a game by ID
    async fn get_game(&self, game_id: GameId) -> Result<Game, GameError> {
        let games = self.games.read().await;
        games.get(&game_id)
            .cloned()
            .ok_or(GameError::GameNotFound)
    }

    /// Create a new game with the given players and broadcast GameStarting message
    pub async fn create_game(&self, players: Vec<PlayerId>) -> GameId {
        // Generate unique game ID using UUID v4
        let game_id = Uuid::new_v4();
        let game_state = GameState::new(players.clone());

        let game = Game {
            id: game_id,
            state: game_state,
            players: players.clone(),
            created_at: Instant::now(),
        };

        let mut games = self.games.write().await;
        games.insert(game_id, game);
        drop(games); // Release lock before broadcasting

        info!("Game {} created with {} players", game_id, players.len());

        // Broadcast GameStarting message to all players
        let msg = ServerMessage::GameStarting { game_id };
        self.connection_manager.broadcast_to_players(&players, msg).await;

        game_id
    }

    /// End a game and remove it from storage
    pub async fn end_game(&self, game_id: GameId) {
        let mut games = self.games.write().await;
        if games.remove(&game_id).is_some() {
            info!("Game {} ended and removed", game_id);
        } else {
            warn!("Attempted to end non-existent game {}", game_id);
        }
    }

    /// Get the game state view for a specific player
    pub async fn get_game_state(&self, game_id: GameId, player_id: PlayerId) -> Result<PlayerGameView, GameError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or(GameError::GameNotFound)?;
        
        // Check if player is in the game
        if !game.players.contains(&player_id) {
            return Err(GameError::PlayerNotInGame);
        }

        Ok(game.state.get_player_view(player_id, game_id))
    }

    /// Handle a player action (bid or card play)
    /// Errors are isolated to this specific game and won't affect other games
    pub async fn handle_player_action(
        &self,
        game_id: GameId,
        player_id: PlayerId,
        action: PlayerAction,
    ) -> Result<(), GameError> {
        // Cancel the turn timer since player acted
        self.cancel_turn_timer(game_id).await;

        // Get mutable access to the game
        // Using a scoped lock ensures other games can be accessed concurrently
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or(GameError::GameNotFound)?;

        // Check if player is in the game
        if !game.players.contains(&player_id) {
            return Err(GameError::PlayerNotInGame);
        }

        // Validate the action before applying
        // Any validation errors are caught and returned without affecting game state
        game.state.validate_action(player_id, &action)?;

        // Store state before applying action to detect phase changes
        let trick_complete_before = game.state.current_trick.is_complete(game.players.len());

        // Apply the action to update state
        // If this fails, the game state remains unchanged
        game.state.apply_action(player_id, action.clone())?;

        // Get the list of players for broadcasting
        let players = game.players.clone();
        let game_id_copy = game_id;
        let phase_after = game.state.phase;

        // Check if trick was just completed
        let trick_just_completed = !trick_complete_before && 
            (phase_after == crate::game_state::GamePhase::RoundComplete || 
             phase_after == crate::game_state::GamePhase::GameComplete ||
             game.state.current_trick.cards.is_empty());

        // Get trick winner and final scores if needed
        let trick_winner = if trick_just_completed && !game.state.completed_tricks.is_empty() {
            Some(game.state.completed_tricks.last().unwrap().winner)
        } else {
            None
        };

        let final_scores = if phase_after == crate::game_state::GamePhase::GameComplete {
            Some(game.state.total_scores.clone())
        } else {
            None
        };

        // Release the write lock before broadcasting
        drop(games);

        debug!("Player {} performed action in game {}", player_id, game_id_copy);

        // Broadcast PlayerAction message to all players
        let action_msg = ServerMessage::PlayerAction {
            player_id,
            action,
        };
        self.connection_manager.broadcast_to_players(&players, action_msg).await;

        // Broadcast TrickComplete when trick finishes
        if let Some(winner) = trick_winner {
            let trick_msg = ServerMessage::TrickComplete {
                winner,
                points: 0, // GBridge doesn't use points per trick
            };
            self.connection_manager.broadcast_to_players(&players, trick_msg).await;
            info!("Trick completed in game {}, winner: {}", game_id_copy, winner);
        }

        // Broadcast GameOver when game ends
        if let Some(scores) = final_scores {
            let game_over_msg = ServerMessage::GameOver {
                final_scores: scores,
            };
            self.connection_manager.broadcast_to_players(&players, game_over_msg).await;
            info!("Game {} completed", game_id_copy);
        }

        Ok(())
    }

    /// Start a turn timer for the current player in a game
    pub async fn start_turn_timer(&self, game_id: GameId, timeout_secs: u64) {
        // Cancel any existing timer for this game
        self.cancel_turn_timer(game_id).await;

        // Get the current player and deadline
        let (current_player, deadline) = {
            let mut games = self.games.write().await;
            if let Some(game) = games.get_mut(&game_id) {
                game.state.set_turn_deadline(timeout_secs);
                (game.state.current_player, game.state.turn_deadline)
            } else {
                return; // Game not found
            }
        };

        let Some(deadline) = deadline else {
            return;
        };

        // Clone Arc references for the async task
        let games = Arc::clone(&self.games);
        let connection_manager = Arc::clone(&self.connection_manager);
        let timer_handles = Arc::clone(&self.timer_handles);

        // Spawn a task to monitor the deadline
        let handle = tokio::spawn(async move {
            // Sleep until the deadline
            tokio::time::sleep_until(deadline.into()).await;

            // Check if the game still exists and the turn hasn't changed
            let auto_action = {
                let games_read = games.read().await;
                if let Some(game) = games_read.get(&game_id) {
                    // Check if it's still the same player's turn and deadline hasn't been updated
                    if game.state.current_player == current_player && game.state.is_turn_expired() {
                        game.state.get_auto_action()
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            // If we have an auto action, apply it
            if let Some(action) = auto_action {
                info!("Turn timeout for player {} in game {}, applying auto action", current_player, game_id);
                
                // Apply the auto action
                let mut games_write = games.write().await;
                if let Some(game) = games_write.get_mut(&game_id) {
                    if let Err(e) = game.state.apply_action(current_player, action.clone()) {
                        warn!("Failed to apply auto action for player {} in game {}: {}", current_player, game_id, e);
                        return;
                    }

                    let players = game.players.clone();
                    drop(games_write);

                    // Broadcast the auto action
                    let action_msg = ServerMessage::PlayerAction {
                        player_id: current_player,
                        action,
                    };
                    connection_manager.broadcast_to_players(&players, action_msg).await;
                }
            }

            // Remove this timer handle
            let mut handles = timer_handles.write().await;
            handles.remove(&game_id);
        });

        // Store the handle so we can cancel it later
        let mut handles = self.timer_handles.write().await;
        handles.insert(game_id, handle);
    }

    /// Cancel the turn timer for a game
    pub async fn cancel_turn_timer(&self, game_id: GameId) {
        let mut handles = self.timer_handles.write().await;
        if let Some(handle) = handles.remove(&game_id) {
            handle.abort();
            debug!("Cancelled turn timer for game {}", game_id);
        }
    }

    /// Get game statistics
    pub async fn get_stats(&self) -> GameStats {
        let games = self.games.read().await;
        let active_games = games.len();

        GameStats {
            active_games,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GameStats {
    pub active_games: usize,
}

// Make Game cloneable for the helper method
impl Clone for Game {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            state: GameState::new(self.players.clone()), // Create new state with same players
            players: self.players.clone(),
            created_at: self.created_at,
        }
    }
}
