use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::connection::PlayerId;
use crate::lobby::LobbyId;
use crate::game::GameId;
use crate::game_logic::card::{Card, Suit};
use crate::game_logic::bidding::Bid;
use crate::game_state::GamePhase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub player_count: usize,
    pub turn_timeout_secs: u64,
    pub allow_reconnect: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            player_count: 4,
            turn_timeout_secs: 30,
            allow_reconnect: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerAction {
    Bid(Bid),
    PlayCard(Card),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResult {
    pub round_number: usize,
    pub player_results: Vec<PlayerRoundResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRoundResult {
    pub player_id: PlayerId,
    pub bid: u8,
    pub tricks_won: u8,
    pub score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGameView {
    pub game_id: GameId,
    pub phase: GamePhase,
    pub your_hand: Vec<Card>,
    pub current_trick: Vec<(PlayerId, Card)>,
    pub scores: HashMap<PlayerId, i32>,
    pub history: Vec<RoundResult>, // Added history
    pub round_number: usize,       // Added round_number
    pub trump_suit: Option<Suit>,
    pub current_player: PlayerId,
    pub your_turn: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyInfo {
    pub id: LobbyId,
    pub host: PlayerId,
    pub players: Vec<PlayerInfo>,
    pub max_players: usize,
    pub settings: GameSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    // Lobby actions
    CreateLobby { settings: GameSettings },
    JoinLobby { lobby_id: LobbyId },
    LeaveLobby,
    StartGame,
    StartNextRound, // Added manual transition
    ListLobbies,

    // Game actions
    PlaceBid { bid: Bid },
    PlayCard { card: Card },
    RequestGameState,

    // Connection
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    // Connection
    Connected { player_id: PlayerId },
    Pong,
    Error { message: String },

    // Lobby updates
    LobbyCreated { lobby_id: LobbyId },
    LobbyJoined { lobby: LobbyInfo },
    LobbyUpdated { lobby: LobbyInfo },
    LobbyList { lobbies: Vec<LobbyInfo> },
    GameStarting { game_id: GameId },

    // Game updates
    GameState { state: PlayerGameView },
    YourTurn { valid_actions: Vec<PlayerAction> },
    PlayerAction { player_id: PlayerId, action: PlayerAction, next_player: PlayerId },
    TrickComplete { winner: PlayerId },
    GameOver { final_scores: HashMap<PlayerId, i32> },

    // Player updates
    PlayerJoined { player_id: PlayerId },
    PlayerLeft { player_id: PlayerId },
    PlayerReconnected { player_id: PlayerId },
}
