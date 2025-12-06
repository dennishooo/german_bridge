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
        // Set lead suit on first card
        if self.cards.is_empty() {
            self.lead_suit = Some(card.suit);
        }
        self.cards.push((player_id, card));
    }

    pub fn is_complete(&self, num_players: usize) -> bool {
        self.cards.len() == num_players
    }

    pub fn winner(&self, trump: Option<Suit>) -> Option<PlayerId> {
        if self.cards.is_empty() {
            return None;
        }

        let lead_suit = self.lead_suit?;
        let (ref winner_id, mut winning_card) = self.cards[0];
        let mut winner_id = winner_id.clone();

        for &(ref player_id, card) in &self.cards[1..] {
            if card.beats(&winning_card, trump, lead_suit) {
                winner_id = player_id.clone();
                winning_card = card;
            }
        }

        Some(winner_id)
    }
}

pub struct CompletedTrick {
    pub winner: PlayerId,
    pub cards: Vec<(PlayerId, Card)>,
    pub points: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_logic::card::{Card, Rank, Suit};

    #[test]
    fn test_add_card_sets_lead_suit() {
        let mut trick = Trick::new();
        let player1 = PlayerId::new_v4();
        let card = Card::new(Suit::Hearts, Rank::Ace);

        trick.add_card(player1, card);

        assert_eq!(trick.lead_suit, Some(Suit::Hearts));
        assert_eq!(trick.cards.len(), 1);
    }

    #[test]
    fn test_add_card_preserves_lead_suit() {
        let mut trick = Trick::new();
        let player1 = PlayerId::new_v4();
        let player2 = PlayerId::new_v4();
        let card1 = Card::new(Suit::Hearts, Rank::Ace);
        let card2 = Card::new(Suit::Spades, Rank::King);

        trick.add_card(player1, card1);
        trick.add_card(player2, card2);

        assert_eq!(trick.lead_suit, Some(Suit::Hearts));
        assert_eq!(trick.cards.len(), 2);
    }

    #[test]
    fn test_is_complete() {
        let mut trick = Trick::new();
        let player1 = PlayerId::new_v4();
        let player2 = PlayerId::new_v4();
        let player3 = PlayerId::new_v4();

        assert!(!trick.is_complete(3));

        trick.add_card(player1, Card::new(Suit::Hearts, Rank::Ace));
        assert!(!trick.is_complete(3));

        trick.add_card(player2, Card::new(Suit::Hearts, Rank::King));
        assert!(!trick.is_complete(3));

        trick.add_card(player3, Card::new(Suit::Hearts, Rank::Queen));
        assert!(trick.is_complete(3));
    }

    #[test]
    fn test_winner_empty_trick() {
        let trick = Trick::new();
        assert_eq!(trick.winner(None), None);
    }

    #[test]
    fn test_winner_single_card() {
        let mut trick = Trick::new();
        let player1 = PlayerId::new_v4();
        let card = Card::new(Suit::Hearts, Rank::Ace);

        trick.add_card(player1, card);

        assert_eq!(trick.winner(None), Some(player1));
    }

    #[test]
    fn test_winner_same_suit_higher_rank_wins() {
        let mut trick = Trick::new();
        let player1 = PlayerId::new_v4();
        let player2 = PlayerId::new_v4();
        let player3 = PlayerId::new_v4();

        trick.add_card(player1, Card::new(Suit::Hearts, Rank::Ten));
        trick.add_card(player2, Card::new(Suit::Hearts, Rank::Ace));
        trick.add_card(player3, Card::new(Suit::Hearts, Rank::King));

        assert_eq!(trick.winner(None), Some(player2)); // Ace wins
    }

    #[test]
    fn test_winner_trump_beats_lead_suit() {
        let mut trick = Trick::new();
        let player1 = PlayerId::new_v4();
        let player2 = PlayerId::new_v4();
        let player3 = PlayerId::new_v4();

        trick.add_card(player1, Card::new(Suit::Hearts, Rank::Ace));
        trick.add_card(player2, Card::new(Suit::Clubs, Rank::Two)); // Trump
        trick.add_card(player3, Card::new(Suit::Hearts, Rank::King));

        assert_eq!(trick.winner(Some(Suit::Clubs)), Some(player2)); // Trump 2 beats Ace
    }

    #[test]
    fn test_winner_lead_suit_beats_non_trump_non_lead() {
        let mut trick = Trick::new();
        let player1 = PlayerId::new_v4();
        let player2 = PlayerId::new_v4();
        let player3 = PlayerId::new_v4();

        trick.add_card(player1, Card::new(Suit::Hearts, Rank::Two));
        trick.add_card(player2, Card::new(Suit::Spades, Rank::Ace)); // Not trump, not lead
        trick.add_card(player3, Card::new(Suit::Diamonds, Rank::Ace)); // Not trump, not lead

        assert_eq!(trick.winner(Some(Suit::Clubs)), Some(player1)); // Lead suit wins
    }

    #[test]
    fn test_winner_higher_trump_wins() {
        let mut trick = Trick::new();
        let player1 = PlayerId::new_v4();
        let player2 = PlayerId::new_v4();
        let player3 = PlayerId::new_v4();

        trick.add_card(player1, Card::new(Suit::Hearts, Rank::Ace));
        trick.add_card(player2, Card::new(Suit::Clubs, Rank::Two)); // Trump
        trick.add_card(player3, Card::new(Suit::Clubs, Rank::King)); // Higher trump

        assert_eq!(trick.winner(Some(Suit::Clubs)), Some(player3)); // King of trump wins
    }

    #[test]
    fn test_winner_complex_scenario() {
        let mut trick = Trick::new();
        let player1 = PlayerId::new_v4();
        let player2 = PlayerId::new_v4();
        let player3 = PlayerId::new_v4();
        let player4 = PlayerId::new_v4();

        // Lead: Hearts 10
        trick.add_card(player1, Card::new(Suit::Hearts, Rank::Ten));
        // Follow: Hearts Ace (higher)
        trick.add_card(player2, Card::new(Suit::Hearts, Rank::Ace));
        // Discard: Spades King (not trump, not lead)
        trick.add_card(player3, Card::new(Suit::Spades, Rank::King));
        // Trump: Diamonds 3 (trump beats all)
        trick.add_card(player4, Card::new(Suit::Diamonds, Rank::Three));

        assert_eq!(trick.winner(Some(Suit::Diamonds)), Some(player4)); // Trump wins
    }
}
