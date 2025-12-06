use crate::connection::PlayerId;
use crate::game_logic::card::{Card, Suit};

pub struct Trick {
    pub lead_suit: Option<Suit>,
    pub cards: Vec<(PlayerId, Card)>,
}

impl Trick {
    pub fn new() -> Self {
        Self {
            lead_suit: None,
            cards: Vec::new(),
        }
    }

    pub fn add_card(&mut self, player_id: PlayerId, card: Card) {
        // TODO: Implement add card logic
    }

    pub fn is_complete(&self, num_players: usize) -> bool {
        // TODO: Implement completion check
        false
    }

    pub fn winner(&self, trump: Option<Suit>) -> Option<PlayerId> {
        // TODO: Implement winner determination
        None
    }
}

pub struct CompletedTrick {
    pub winner: PlayerId,
    pub cards: Vec<(PlayerId, Card)>,
    pub points: u8,
}
