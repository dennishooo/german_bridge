use std::collections::HashMap;
use std::time::Instant;
use crate::connection::PlayerId;

pub struct GameState {
    pub phase: GamePhase,
    pub current_player: PlayerId,
    pub turn_deadline: Option<Instant>,
}

pub enum GamePhase {
    Bidding,
    Playing,
    Finished,
}

impl GameState {
    pub fn new(players: Vec<PlayerId>) -> Self {
        Self {
            phase: GamePhase::Bidding,
            current_player: players[0],
            turn_deadline: None,
        }
    }
}
