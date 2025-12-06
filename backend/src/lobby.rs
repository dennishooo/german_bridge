use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::connection::PlayerId;
use crate::protocol::GameSettings;
use crate::game::{GameManager, GameId};
use tracing::{debug, info, warn};

pub type LobbyId = Uuid;

pub struct LobbyManager {
    lobbies: Arc<RwLock<HashMap<LobbyId, Lobby>>>,
    game_manager: Arc<GameManager>,
    connection_manager: Arc<crate::connection::ConnectionManager>,
}

#[derive(Clone)]
pub struct Lobby {
    pub id: LobbyId,
    pub host: PlayerId,
    pub players: Vec<PlayerId>,
    pub max_players: usize,
    pub created_at: Instant,
    pub settings: GameSettings,
}

impl Lobby {
    /// Check if the lobby is full
    pub fn is_full(&self) -> bool {
        self.players.len() >= self.max_players
    }

    /// Check if the given player is the host
    pub fn is_host(&self, player_id: PlayerId) -> bool {
        self.host == player_id
    }
}

impl LobbyManager {
    pub fn new(game_manager: Arc<GameManager>, connection_manager: Arc<crate::connection::ConnectionManager>) -> Self {
        Self {
            lobbies: Arc::new(RwLock::new(HashMap::new())),
            game_manager,
            connection_manager,
        }
    }

    /// Create a new lobby with the given host and settings
    pub async fn create_lobby(&self, host: PlayerId, settings: GameSettings) -> LobbyId {
        let lobby_id = Uuid::new_v4();
        let max_players = match settings.player_count {
            crate::protocol::PlayerCount::Three => 3,
            crate::protocol::PlayerCount::Four => 4,
        };

        let lobby = Lobby {
            id: lobby_id,
            host: host.clone(),
            players: vec![host.clone()],
            max_players,
            created_at: Instant::now(),
            settings,
        };

        let mut lobbies = self.lobbies.write().await;
        lobbies.insert(lobby_id, lobby);

        info!("Lobby {} created by player {} with max {} players", lobby_id, host, max_players);

        lobby_id
    }

    /// Join an existing lobby
    pub async fn join_lobby(&self, lobby_id: LobbyId, player_id: PlayerId) -> Result<(), crate::error::LobbyError> {
        let mut lobbies = self.lobbies.write().await;
        
        let lobby = lobbies.get_mut(&lobby_id)
            .ok_or(crate::error::LobbyError::LobbyNotFound)?;

        if lobby.is_full() {
            warn!("Player {} attempted to join full lobby {}", player_id, lobby_id);
            return Err(crate::error::LobbyError::LobbyFull);
        }

        // Don't add if already in lobby
        if !lobby.players.contains(&player_id) {
            lobby.players.push(player_id.clone());
            info!("Player {} joined lobby {} ({}/{} players)", player_id, lobby_id, lobby.players.len(), lobby.max_players);
        } else {
            debug!("Player {} already in lobby {}", player_id, lobby_id);
        }

        Ok(())
    }

    /// Leave a lobby, with host transfer if necessary
    pub async fn leave_lobby(&self, lobby_id: LobbyId, player_id: PlayerId) -> Result<(), crate::error::LobbyError> {
        let mut lobbies = self.lobbies.write().await;
        
        let lobby = lobbies.get_mut(&lobby_id)
            .ok_or(crate::error::LobbyError::LobbyNotFound)?;

        // Remove player from lobby
        lobby.players.retain(|p| *p != player_id);
        info!("Player {} left lobby {}", player_id, lobby_id);

        // If lobby is empty, remove it
        if lobby.players.is_empty() {
            lobbies.remove(&lobby_id);
            info!("Lobby {} removed (empty)", lobby_id);
            return Ok(());
        }

        // If the host left, transfer to next player
        if lobby.host == player_id {
            let new_host = lobby.players[0].clone();
            lobby.host = new_host.clone();
            info!("Lobby {} host transferred from {} to {}", lobby_id, player_id, new_host);
        }

        Ok(())
    }

    /// List all joinable lobbies
    pub async fn list_lobbies(&self) -> Vec<crate::protocol::LobbyInfo> {
        let lobbies = self.lobbies.read().await;
        
        let mut joinable_lobbies = Vec::new();
        for lobby in lobbies.values().filter(|lobby| !lobby.is_full()) {
            // Build Vec<PlayerInfo>
            let mut players = Vec::new();
            for player_id in &lobby.players {
                if let Some(username) = self.connection_manager.get_username(player_id).await {
                    players.push(crate::protocol::PlayerInfo {
                        id: player_id.clone(),
                        username,
                    });
                }
            }
            
            joinable_lobbies.push(crate::protocol::LobbyInfo {
                id: lobby.id,
                host: lobby.host.clone(),
                players,
                max_players: lobby.max_players,
                settings: lobby.settings.clone(),
            });
        }
        
        debug!("Listing {} joinable lobbies", joinable_lobbies.len());
        joinable_lobbies
    }

    /// Get a lobby by ID (helper method)
    pub async fn get_lobby(&self, lobby_id: LobbyId) -> Option<Lobby> {
        let lobbies = self.lobbies.read().await;
        lobbies.get(&lobby_id).cloned()
    }

    /// Start a game from a lobby
    pub async fn start_game(&self, lobby_id: LobbyId, caller: PlayerId) -> Result<GameId, crate::error::LobbyError> {
        // Get lobby info before removing it
        let players = {
            let lobbies = self.lobbies.read().await;
            let lobby = lobbies.get(&lobby_id)
                .ok_or(crate::error::LobbyError::LobbyNotFound)?;

            // Verify caller is host
            if !lobby.is_host(caller.clone()) {
                warn!("Player {} attempted to start game in lobby {} but is not host", caller, lobby_id);
                return Err(crate::error::LobbyError::NotHost);
            }

            // Validate player count (2+ players)
            if lobby.players.len() < 2 {
                warn!("Lobby {} cannot start game with only {} players", lobby_id, lobby.players.len());
                return Err(crate::error::LobbyError::NotEnoughPlayers);
            }

            lobby.players.clone()
        };

        info!("Starting game from lobby {} with {} players", lobby_id, players.len());

        // Create the game
        let game_id = self.game_manager.create_game(players).await;

        // Remove the lobby after game starts
        let mut lobbies = self.lobbies.write().await;
        lobbies.remove(&lobby_id);
        info!("Lobby {} removed after game {} started", lobby_id, game_id);

        Ok(game_id)
    }
}
