use serde::{Deserialize, Serialize};
use crate::connection::PlayerId;
use crate::error::GameError;

pub struct BiddingState {
    pub bids: Vec<(PlayerId, Bid)>,
    pub current_bidder: PlayerId,
    pub highest_bid: Option<Bid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Bid {
    Pass,
    Game(u8),
    Schneider,
    Schwarz,
}

impl BiddingState {
    pub fn new(starting_player: PlayerId) -> Self {
        Self {
            bids: Vec::new(),
            current_bidder: starting_player,
            highest_bid: None,
        }
    }

    pub fn place_bid(&mut self, player_id: PlayerId, bid: Bid) -> Result<(), GameError> {
        // TODO: Implement bid placement logic
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        // TODO: Implement completion check
        false
    }

    pub fn determine_declarer(&self) -> Option<PlayerId> {
        // TODO: Implement declarer determination
        None
    }
}
