use std::sync::Arc;
use crate::connection::{ConnectionManager, PlayerId};
use crate::lobby::LobbyManager;
use crate::game::GameManager;
use crate::protocol::ClientMessage;
use crate::error::RouterError;

pub struct MessageRouter {
    lobby_manager: Arc<LobbyManager>,
    game_manager: Arc<GameManager>,
    connection_manager: Arc<ConnectionManager>,
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
        }
    }

    pub async fn route_message(
        &self,
        player_id: PlayerId,
        message: ClientMessage,
    ) -> Result<(), RouterError> {
        // TODO: Implement message routing
        Ok(())
    }
}
