use serde::{Deserialize, Serialize};
use crate::connection::PlayerId;
use crate::lobby::LobbyId;
use crate::game::GameId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    // Lobby actions
    CreateLobby,
    JoinLobby { lobby_id: LobbyId },
    LeaveLobby,
    StartGame,
    ListLobbies,

    // Game actions
    PlaceBid,
    PlayCard,
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
    GameStarting { game_id: GameId },

    // Game updates
    GameOver,
}
