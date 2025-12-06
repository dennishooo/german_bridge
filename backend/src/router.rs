use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::connection::{ConnectionManager, PlayerId};
use crate::lobby::{LobbyManager, LobbyId};
use crate::game::{GameManager, GameId};
use crate::protocol::{ClientMessage, ServerMessage, PlayerAction};
use crate::error::RouterError;
use tracing::{debug, error, info};

pub struct MessageRouter {
    lobby_manager: Arc<LobbyManager>,
    game_manager: Arc<GameManager>,
    connection_manager: Arc<ConnectionManager>,
    player_to_game: Arc<RwLock<HashMap<PlayerId, GameId>>>,
    player_to_lobby: Arc<RwLock<HashMap<PlayerId, LobbyId>>>,
}

impl MessageRouter {
    pub fn new(
        lobby_manager: Arc<LobbyManager>,
        game_manager: Arc<GameManager>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        Self {
            lobby_manager,
            game_manager,
            connection_manager,
            player_to_game: Arc::new(RwLock::new(HashMap::new())),
            player_to_lobby: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn route_message(
        &self,
        player_id: PlayerId,
        message: ClientMessage,
    ) -> Result<(), RouterError> {
        debug!("Routing message from player {}: {:?}", player_id, message);

        // Match on ClientMessage variants and route to appropriate handlers
        // Each handler is isolated and errors won't affect other games
        let result = match message {
            // Lobby message handlers
            ClientMessage::CreateLobby { settings } => {
                self.handle_create_lobby(player_id, settings).await
            }
            ClientMessage::JoinLobby { lobby_id } => {
                self.handle_join_lobby(player_id, lobby_id).await
            }
            ClientMessage::LeaveLobby => {
                self.handle_leave_lobby(player_id).await
            }
            ClientMessage::StartGame => {
                self.handle_start_game(player_id).await
            }
            ClientMessage::ListLobbies => {
                self.handle_list_lobbies(player_id).await
            }

            // Game message handlers
            ClientMessage::PlaceBid { bid } => {
                self.handle_place_bid(player_id, bid).await
            }
            ClientMessage::PlayCard { card } => {
                self.handle_play_card(player_id, card).await
            }
            ClientMessage::RequestGameState => {
                self.handle_request_game_state(player_id).await
            }

            // Connection message handlers
            ClientMessage::Ping => {
                self.handle_ping(player_id).await
            }
        };

        // Convert errors to ServerMessage::Error and send to client
        // This ensures errors are logged and communicated without crashing
        if let Err(e) = &result {
            error!("Error routing message from player {}: {}", player_id, e);
            let error_msg = ServerMessage::Error {
                message: e.to_string(),
            };
            self.connection_manager.send_to_player(player_id, error_msg).await;
        }

        result
    }

    // Lobby message handlers

    async fn handle_create_lobby(
        &self,
        player_id: PlayerId,
        settings: crate::protocol::GameSettings,
    ) -> Result<(), RouterError> {
        info!("Player {} creating lobby", player_id);
        
        let lobby_id = self.lobby_manager.create_lobby(player_id, settings).await;
        
        // Track player-to-lobby mapping
        let mut player_to_lobby = self.player_to_lobby.write().await;
        player_to_lobby.insert(player_id, lobby_id);
        drop(player_to_lobby);
        
        let msg = ServerMessage::LobbyCreated { lobby_id };
        self.connection_manager.send_to_player(player_id, msg).await;
        
        Ok(())
    }

    async fn handle_join_lobby(
        &self,
        player_id: PlayerId,
        lobby_id: crate::lobby::LobbyId,
    ) -> Result<(), RouterError> {
        info!("Player {} joining lobby {}", player_id, lobby_id);
        
        self.lobby_manager.join_lobby(lobby_id, player_id).await?;
        
        // Track player-to-lobby mapping
        let mut player_to_lobby = self.player_to_lobby.write().await;
        player_to_lobby.insert(player_id, lobby_id);
        drop(player_to_lobby);
        
        // Get lobby info to send back
        if let Some(lobby) = self.lobby_manager.get_lobby(lobby_id).await {
            let lobby_info = crate::protocol::LobbyInfo {
                id: lobby.id,
                host: lobby.host,
                players: lobby.players.clone(),
                max_players: lobby.max_players,
                settings: lobby.settings.clone(),
            };
            
            let msg = ServerMessage::LobbyJoined { lobby: lobby_info };
            self.connection_manager.send_to_player(player_id, msg).await;
        }
        
        Ok(())
    }

    async fn handle_leave_lobby(
        &self,
        player_id: PlayerId,
    ) -> Result<(), RouterError> {
        info!("Player {} leaving lobby", player_id);
        
        // Get the lobby ID from the mapping
        let lobby_id = {
            let player_to_lobby = self.player_to_lobby.read().await;
            player_to_lobby.get(&player_id).copied()
        };
        
        if let Some(lobby_id) = lobby_id {
            self.lobby_manager.leave_lobby(lobby_id, player_id).await?;
            
            // Remove from mapping
            let mut player_to_lobby = self.player_to_lobby.write().await;
            player_to_lobby.remove(&player_id);
        }
        
        Ok(())
    }

    async fn handle_start_game(
        &self,
        player_id: PlayerId,
    ) -> Result<(), RouterError> {
        info!("Player {} starting game", player_id);
        
        // Get the lobby ID from the mapping
        let lobby_id = {
            let player_to_lobby = self.player_to_lobby.read().await;
            player_to_lobby.get(&player_id).copied()
        };
        
        if let Some(lobby_id) = lobby_id {
            // Get all players in the lobby before starting
            let players = if let Some(lobby) = self.lobby_manager.get_lobby(lobby_id).await {
                lobby.players.clone()
            } else {
                return Err(crate::error::LobbyError::LobbyNotFound.into());
            };
            
            let game_id = self.lobby_manager.start_game(lobby_id, player_id).await?;
            
            // Update mappings: remove from lobby, add to game
            let mut player_to_lobby = self.player_to_lobby.write().await;
            let mut player_to_game = self.player_to_game.write().await;
            
            for player in &players {
                player_to_lobby.remove(player);
                player_to_game.insert(*player, game_id);
            }
            
            // GameStarting message is already sent by GameManager::create_game
            info!("Game {} started from lobby {}", game_id, lobby_id);
            Ok(())
        } else {
            // Player is not in any lobby
            Err(crate::error::LobbyError::NotHost.into())
        }
    }

    async fn handle_list_lobbies(
        &self,
        player_id: PlayerId,
    ) -> Result<(), RouterError> {
        debug!("Player {} requesting lobby list", player_id);
        
        let lobbies = self.lobby_manager.list_lobbies().await;
        
        let msg = ServerMessage::LobbyList { lobbies };
        self.connection_manager.send_to_player(player_id, msg).await;
        
        Ok(())
    }

    // Game message handlers

    async fn handle_place_bid(
        &self,
        player_id: PlayerId,
        bid: crate::game_logic::bidding::Bid,
    ) -> Result<(), RouterError> {
        info!("Player {} placing bid: {:?}", player_id, bid);
        
        // Get the game ID from the mapping
        let game_id = {
            let player_to_game = self.player_to_game.read().await;
            player_to_game.get(&player_id).copied()
                .ok_or(crate::error::GameError::GameNotFound)?
        };
        
        let action = PlayerAction::Bid(bid);
        self.game_manager.handle_player_action(game_id, player_id, action).await?;
        
        Ok(())
    }

    async fn handle_play_card(
        &self,
        player_id: PlayerId,
        card: crate::game_logic::card::Card,
    ) -> Result<(), RouterError> {
        info!("Player {} playing card: {:?}", player_id, card);
        
        // Get the game ID from the mapping
        let game_id = {
            let player_to_game = self.player_to_game.read().await;
            player_to_game.get(&player_id).copied()
                .ok_or(crate::error::GameError::GameNotFound)?
        };
        
        let action = PlayerAction::PlayCard(card);
        self.game_manager.handle_player_action(game_id, player_id, action).await?;
        
        Ok(())
    }

    async fn handle_request_game_state(
        &self,
        player_id: PlayerId,
    ) -> Result<(), RouterError> {
        debug!("Player {} requesting game state", player_id);
        
        // Get the game ID from the mapping
        let game_id = {
            let player_to_game = self.player_to_game.read().await;
            player_to_game.get(&player_id).copied()
                .ok_or(crate::error::GameError::GameNotFound)?
        };
        
        let state = self.game_manager.get_game_state(game_id, player_id).await?;
        
        let msg = ServerMessage::GameState { state };
        self.connection_manager.send_to_player(player_id, msg).await;
        
        Ok(())
    }

    // Connection message handlers

    async fn handle_ping(
        &self,
        player_id: PlayerId,
    ) -> Result<(), RouterError> {
        debug!("Player {} sent ping", player_id);
        
        let msg = ServerMessage::Pong;
        self.connection_manager.send_to_player(player_id, msg).await;
        
        Ok(())
    }
}
