use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Invalid move: {0}")]
    InvalidMove(String),

    #[error("Not player's turn")]
    NotPlayerTurn,

    #[error("Game not found")]
    GameNotFound,

    #[error("Player not in game")]
    PlayerNotInGame,
}

#[derive(Debug, Error)]
pub enum LobbyError {
    #[error("Lobby full")]
    LobbyFull,

    #[error("Lobby not found")]
    LobbyNotFound,

    #[error("Not enough players")]
    NotEnoughPlayers,

    #[error("Only host can start game")]
    NotHost,
}

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Game error: {0}")]
    Game(#[from] GameError),

    #[error("Lobby error: {0}")]
    Lobby(#[from] LobbyError),

    #[error("Unknown message type")]
    UnknownMessage,

    #[error("{0}")]
    Generic(String),
}

impl From<String> for RouterError {
    fn from(s: String) -> Self {
        RouterError::Generic(s)
    }
}

impl From<&str> for RouterError {
    fn from(s: &str) -> Self {
        RouterError::Generic(s.to_string())
    }
}
