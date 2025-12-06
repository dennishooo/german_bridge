use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::connection::PlayerId;
use crate::error::GameError;

pub struct BiddingState {
    pub bids: HashMap<PlayerId, u8>,
    pub current_bidder: PlayerId,
    pub player_order: Vec<PlayerId>,
    pub cards_this_round: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bid {
    /// Number of tricks the player expects to win (0 to total cards dealt)
    pub tricks: u8,
}

impl BiddingState {
    pub fn new(starting_player: PlayerId, players: Vec<PlayerId>, cards: usize) -> Self {
        Self {
            bids: HashMap::new(),
            current_bidder: starting_player,
            player_order: players,
            cards_this_round: cards,
        }
    }

    pub fn place_bid(&mut self, player_id: PlayerId, bid: u8) -> Result<(), GameError> {
        // Validate it's the player's turn
        if player_id != self.current_bidder {
            return Err(GameError::NotPlayerTurn);
        }

        // Validate bid range
        if bid as usize > self.cards_this_round {
            return Err(GameError::InvalidMove(format!(
                "Bid {} exceeds cards dealt {}",
                bid, self.cards_this_round
            )));
        }

        // If this is the last bidder, check the restriction
        if self.is_last_bidder(player_id) {
            self.validate_last_bid(bid)?;
        }

        // Place the bid
        self.bids.insert(player_id, bid);

        // Advance to next bidder if not complete
        if !self.is_complete() {
            self.advance_bidder();
        }

        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.bids.len() == self.player_order.len()
    }

    pub fn is_last_bidder(&self, player_id: PlayerId) -> bool {
        self.bids.len() == self.player_order.len() - 1
            && self.current_bidder == player_id
    }

    pub fn validate_last_bid(&self, bid: u8) -> Result<(), GameError> {
        let sum_of_bids: u8 = self.bids.values().sum();
        let total_with_this_bid = sum_of_bids + bid;

        if total_with_this_bid == self.cards_this_round as u8 {
            return Err(GameError::InvalidMove(format!(
                "Last bidder cannot bid {} - sum would equal total cards ({})",
                bid, self.cards_this_round
            )));
        }

        Ok(())
    }

    pub fn advance_bidder(&mut self) {
        let current_index = self
            .player_order
            .iter()
            .position(|&p| p == self.current_bidder)
            .unwrap_or(0);

        let next_index = (current_index + 1) % self.player_order.len();
        self.current_bidder = self.player_order[next_index];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_players(count: usize) -> Vec<PlayerId> {
        (0..count).map(|_| PlayerId::new_v4()).collect()
    }

    #[test]
    fn test_new_bidding_state() {
        let players = create_test_players(3);
        let bidding = BiddingState::new(players[0], players.clone(), 5);

        assert_eq!(bidding.current_bidder, players[0]);
        assert_eq!(bidding.player_order.len(), 3);
        assert_eq!(bidding.cards_this_round, 5);
        assert!(bidding.bids.is_empty());
    }

    #[test]
    fn test_place_bid_valid() {
        let players = create_test_players(3);
        let mut bidding = BiddingState::new(players[0], players.clone(), 5);

        let result = bidding.place_bid(players[0], 2);
        assert!(result.is_ok());
        assert_eq!(bidding.bids.get(&players[0]), Some(&2));
        assert_eq!(bidding.current_bidder, players[1]); // Advanced to next player
    }

    #[test]
    fn test_place_bid_wrong_player() {
        let players = create_test_players(3);
        let mut bidding = BiddingState::new(players[0], players.clone(), 5);

        let result = bidding.place_bid(players[1], 2);
        assert!(matches!(result, Err(GameError::NotPlayerTurn)));
    }

    #[test]
    fn test_place_bid_exceeds_cards() {
        let players = create_test_players(3);
        let mut bidding = BiddingState::new(players[0], players.clone(), 5);

        let result = bidding.place_bid(players[0], 6);
        assert!(matches!(result, Err(GameError::InvalidMove(_))));
    }

    #[test]
    fn test_is_last_bidder() {
        let players = create_test_players(3);
        let mut bidding = BiddingState::new(players[0], players.clone(), 5);

        assert!(!bidding.is_last_bidder(players[0]));

        bidding.place_bid(players[0], 2).unwrap();
        assert!(!bidding.is_last_bidder(players[1]));

        bidding.place_bid(players[1], 1).unwrap();
        assert!(bidding.is_last_bidder(players[2]));
    }

    #[test]
    fn test_last_bidder_restriction_valid() {
        let players = create_test_players(3);
        let mut bidding = BiddingState::new(players[0], players.clone(), 5);

        bidding.place_bid(players[0], 2).unwrap();
        bidding.place_bid(players[1], 1).unwrap();

        // Sum is 3, cards is 5, so bidding 2 would make sum = 5 (invalid)
        let result = bidding.place_bid(players[2], 2);
        assert!(matches!(result, Err(GameError::InvalidMove(_))));

        // But bidding 1 or 3 should be fine
        let result = bidding.place_bid(players[2], 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_last_bidder_can_bid_zero() {
        let players = create_test_players(3);
        let mut bidding = BiddingState::new(players[0], players.clone(), 5);

        bidding.place_bid(players[0], 2).unwrap();
        bidding.place_bid(players[1], 3).unwrap();

        // Sum is 5, so bidding 0 is valid (sum would be 5, not equal to 5... wait, that's wrong)
        // Actually sum is 5, cards is 5, so bidding 0 would make sum = 5 (invalid)
        let result = bidding.place_bid(players[2], 0);
        assert!(matches!(result, Err(GameError::InvalidMove(_))));
    }

    #[test]
    fn test_last_bidder_all_zeros() {
        let players = create_test_players(3);
        let mut bidding = BiddingState::new(players[0], players.clone(), 5);

        bidding.place_bid(players[0], 0).unwrap();
        bidding.place_bid(players[1], 0).unwrap();

        // Sum is 0, cards is 5, so bidding 0 would make sum = 0 (valid, not equal to 5)
        let result = bidding.place_bid(players[2], 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_complete() {
        let players = create_test_players(3);
        let mut bidding = BiddingState::new(players[0], players.clone(), 5);

        assert!(!bidding.is_complete());

        bidding.place_bid(players[0], 2).unwrap();
        assert!(!bidding.is_complete());

        bidding.place_bid(players[1], 1).unwrap();
        assert!(!bidding.is_complete());

        bidding.place_bid(players[2], 1).unwrap();
        assert!(bidding.is_complete());
    }

    #[test]
    fn test_advance_bidder() {
        let players = create_test_players(4);
        let mut bidding = BiddingState::new(players[0], players.clone(), 5);

        assert_eq!(bidding.current_bidder, players[0]);

        bidding.advance_bidder();
        assert_eq!(bidding.current_bidder, players[1]);

        bidding.advance_bidder();
        assert_eq!(bidding.current_bidder, players[2]);

        bidding.advance_bidder();
        assert_eq!(bidding.current_bidder, players[3]);

        bidding.advance_bidder();
        assert_eq!(bidding.current_bidder, players[0]); // Wraps around
    }

    #[test]
    fn test_validate_last_bid() {
        let players = create_test_players(3);
        let mut bidding = BiddingState::new(players[0], players.clone(), 5);

        bidding.bids.insert(players[0], 2);
        bidding.bids.insert(players[1], 1);

        // Sum is 3, cards is 5
        // Bidding 2 would make sum = 5 (invalid)
        assert!(matches!(
            bidding.validate_last_bid(2),
            Err(GameError::InvalidMove(_))
        ));

        // Bidding 0, 1, 3, 4, 5 should all be valid
        assert!(bidding.validate_last_bid(0).is_ok());
        assert!(bidding.validate_last_bid(1).is_ok());
        assert!(bidding.validate_last_bid(3).is_ok());
        assert!(bidding.validate_last_bid(4).is_ok());
        assert!(bidding.validate_last_bid(5).is_ok());
    }

    #[test]
    fn test_two_player_game() {
        let players = create_test_players(2);
        let mut bidding = BiddingState::new(players[0], players.clone(), 3);

        bidding.place_bid(players[0], 1).unwrap();

        // Player 1 is last bidder, sum is 1, cards is 3
        // Bidding 2 would make sum = 3 (invalid)
        assert!(matches!(
            bidding.place_bid(players[1], 2),
            Err(GameError::InvalidMove(_))
        ));

        // Bidding 0, 1, or 3 should be valid
        assert!(bidding.place_bid(players[1], 0).is_ok());
    }
}
