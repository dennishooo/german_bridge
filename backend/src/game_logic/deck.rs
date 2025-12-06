use crate::game_logic::card::{Card, Suit, Rank};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    /// Create a standard 52-card deck for German Bridge (2-A in 4 suits)
    pub fn new_german_bridge() -> Self {
        let mut cards = Vec::with_capacity(52);
        let suits = [Suit::Clubs, Suit::Spades, Suit::Hearts, Suit::Diamonds];
        let ranks = [
            Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
            Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
            Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
        ];

        for suit in &suits {
            for rank in &ranks {
                cards.push(Card::new(*suit, *rank));
            }
        }

        Self { cards }
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    /// Deal cards evenly to the specified number of players
    /// Returns a vector of Hands, one for each player
    pub fn deal(&mut self, num_players: usize) -> Vec<Hand> {
        let mut hands = vec![Vec::new(); num_players];
        
        // Deal cards in round-robin fashion
        let mut player_idx = 0;
        while !self.cards.is_empty() {
            if let Some(card) = self.cards.pop() {
                hands[player_idx].push(card);
                player_idx = (player_idx + 1) % num_players;
            }
        }

        hands.into_iter().map(Hand::new).collect()
    }
}

pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn new(cards: Vec<Card>) -> Self {
        Self { cards }
    }

    /// Play a card from the hand
    /// Returns the card if it exists in the hand, otherwise returns an error
    pub fn play_card(&mut self, card: Card) -> Result<Card, crate::error::GameError> {
        if let Some(pos) = self.cards.iter().position(|c| *c == card) {
            Ok(self.cards.remove(pos))
        } else {
            Err(crate::error::GameError::InvalidMove(
                "Card not in hand".to_string()
            ))
        }
    }

    /// Get valid plays based on the lead suit
    /// If lead_suit is None (first card of trick), all cards are valid
    /// If lead_suit is Some, must follow suit if possible
    pub fn valid_plays(&self, lead_suit: Option<Suit>) -> Vec<Card> {
        match lead_suit {
            None => {
                // First card of trick - all cards are valid
                self.cards.clone()
            }
            Some(suit) => {
                // Must follow suit if possible
                let cards_in_suit: Vec<Card> = self.cards
                    .iter()
                    .filter(|c| c.suit == suit)
                    .copied()
                    .collect();
                
                if cards_in_suit.is_empty() {
                    // No cards in lead suit - can play any card
                    self.cards.clone()
                } else {
                    // Must play a card in the lead suit
                    cards_in_suit
                }
            }
        }
    }

    /// Check if the hand contains a specific card
    pub fn has_card(&self, card: &Card) -> bool {
        self.cards.contains(card)
    }

    /// Get a reference to all cards in the hand
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_german_bridge_creates_52_cards() {
        let deck = Deck::new_german_bridge();
        assert_eq!(deck.cards.len(), 52, "Deck should contain 52 cards");
    }

    #[test]
    fn test_new_german_bridge_has_all_suits() {
        let deck = Deck::new_german_bridge();
        
        let clubs_count = deck.cards.iter().filter(|c| c.suit == Suit::Clubs).count();
        let spades_count = deck.cards.iter().filter(|c| c.suit == Suit::Spades).count();
        let hearts_count = deck.cards.iter().filter(|c| c.suit == Suit::Hearts).count();
        let diamonds_count = deck.cards.iter().filter(|c| c.suit == Suit::Diamonds).count();
        
        assert_eq!(clubs_count, 13, "Should have 13 clubs");
        assert_eq!(spades_count, 13, "Should have 13 spades");
        assert_eq!(hearts_count, 13, "Should have 13 hearts");
        assert_eq!(diamonds_count, 13, "Should have 13 diamonds");
    }

    #[test]
    fn test_new_german_bridge_has_all_ranks() {
        let deck = Deck::new_german_bridge();
        
        let ranks = [
            Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
            Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
            Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
        ];
        
        for rank in &ranks {
            let count = deck.cards.iter().filter(|c| c.rank == *rank).count();
            assert_eq!(count, 4, "Should have 4 cards of rank {:?}", rank);
        }
    }

    #[test]
    fn test_shuffle_changes_order() {
        let mut deck1 = Deck::new_german_bridge();
        let original_order = deck1.cards.clone();
        
        deck1.shuffle();
        
        // It's extremely unlikely that shuffle produces the same order
        // (probability is 1/52! which is essentially 0)
        assert_ne!(deck1.cards, original_order, "Shuffle should change card order");
    }

    #[test]
    fn test_shuffle_preserves_cards() {
        let mut deck = Deck::new_german_bridge();
        let original_len = deck.cards.len();
        
        deck.shuffle();
        
        assert_eq!(deck.cards.len(), original_len, "Shuffle should preserve number of cards");
        assert_eq!(deck.cards.len(), 52, "Should still have 52 cards after shuffle");
    }

    #[test]
    fn test_deal_distributes_evenly_to_4_players() {
        let mut deck = Deck::new_german_bridge();
        deck.shuffle();
        
        let hands = deck.deal(4);
        
        assert_eq!(hands.len(), 4, "Should create 4 hands");
        assert_eq!(hands[0].cards.len(), 13, "Each player should get 13 cards");
        assert_eq!(hands[1].cards.len(), 13, "Each player should get 13 cards");
        assert_eq!(hands[2].cards.len(), 13, "Each player should get 13 cards");
        assert_eq!(hands[3].cards.len(), 13, "Each player should get 13 cards");
    }

    #[test]
    fn test_deal_distributes_evenly_to_3_players() {
        let mut deck = Deck::new_german_bridge();
        deck.shuffle();
        
        let hands = deck.deal(3);
        
        assert_eq!(hands.len(), 3, "Should create 3 hands");
        // 52 cards / 3 players = 17 cards each, with 1 card remaining
        assert_eq!(hands[0].cards.len(), 18, "First player gets 18 cards");
        assert_eq!(hands[1].cards.len(), 17, "Second player gets 17 cards");
        assert_eq!(hands[2].cards.len(), 17, "Third player gets 17 cards");
    }

    #[test]
    fn test_deal_empties_deck() {
        let mut deck = Deck::new_german_bridge();
        deck.shuffle();
        
        let _hands = deck.deal(4);
        
        assert_eq!(deck.cards.len(), 0, "Deck should be empty after dealing");
    }

    #[test]
    fn test_deal_no_duplicate_cards() {
        let mut deck = Deck::new_german_bridge();
        deck.shuffle();
        
        let hands = deck.deal(4);
        
        // Collect all cards from all hands
        let mut all_cards: Vec<&Card> = Vec::new();
        for hand in &hands {
            all_cards.extend(hand.cards.iter());
        }
        
        // Check for duplicates
        for i in 0..all_cards.len() {
            for j in (i + 1)..all_cards.len() {
                assert_ne!(
                    all_cards[i], all_cards[j],
                    "Found duplicate card: {:?}",
                    all_cards[i]
                );
            }
        }
    }

    // Hand tests
    #[test]
    fn test_hand_new() {
        let cards = vec![
            Card::new(Suit::Hearts, Rank::Ace),
            Card::new(Suit::Spades, Rank::King),
        ];
        let hand = Hand::new(cards.clone());
        
        assert_eq!(hand.cards().len(), 2);
        assert_eq!(hand.cards()[0], cards[0]);
        assert_eq!(hand.cards()[1], cards[1]);
    }

    #[test]
    fn test_hand_has_card() {
        let ace_hearts = Card::new(Suit::Hearts, Rank::Ace);
        let king_spades = Card::new(Suit::Spades, Rank::King);
        let queen_clubs = Card::new(Suit::Clubs, Rank::Queen);
        
        let cards = vec![ace_hearts, king_spades];
        let hand = Hand::new(cards);
        
        assert!(hand.has_card(&ace_hearts), "Should have ace of hearts");
        assert!(hand.has_card(&king_spades), "Should have king of spades");
        assert!(!hand.has_card(&queen_clubs), "Should not have queen of clubs");
    }

    #[test]
    fn test_hand_cards_getter() {
        let cards = vec![
            Card::new(Suit::Hearts, Rank::Ace),
            Card::new(Suit::Spades, Rank::King),
            Card::new(Suit::Diamonds, Rank::Queen),
        ];
        let hand = Hand::new(cards.clone());
        
        let hand_cards = hand.cards();
        assert_eq!(hand_cards.len(), 3);
        assert_eq!(hand_cards, &cards[..]);
    }

    #[test]
    fn test_play_card_success() {
        let ace_hearts = Card::new(Suit::Hearts, Rank::Ace);
        let king_spades = Card::new(Suit::Spades, Rank::King);
        
        let cards = vec![ace_hearts, king_spades];
        let mut hand = Hand::new(cards);
        
        let result = hand.play_card(ace_hearts);
        assert!(result.is_ok(), "Should successfully play card");
        assert_eq!(result.unwrap(), ace_hearts);
        assert_eq!(hand.cards().len(), 1, "Hand should have 1 card left");
        assert!(!hand.has_card(&ace_hearts), "Should not have ace of hearts anymore");
        assert!(hand.has_card(&king_spades), "Should still have king of spades");
    }

    #[test]
    fn test_play_card_not_in_hand() {
        let ace_hearts = Card::new(Suit::Hearts, Rank::Ace);
        let king_spades = Card::new(Suit::Spades, Rank::King);
        let queen_clubs = Card::new(Suit::Clubs, Rank::Queen);
        
        let cards = vec![ace_hearts, king_spades];
        let mut hand = Hand::new(cards);
        
        let result = hand.play_card(queen_clubs);
        assert!(result.is_err(), "Should fail to play card not in hand");
        assert_eq!(hand.cards().len(), 2, "Hand should still have 2 cards");
    }

    #[test]
    fn test_valid_plays_first_card() {
        let cards = vec![
            Card::new(Suit::Hearts, Rank::Ace),
            Card::new(Suit::Spades, Rank::King),
            Card::new(Suit::Diamonds, Rank::Queen),
        ];
        let hand = Hand::new(cards.clone());
        
        let valid = hand.valid_plays(None);
        assert_eq!(valid.len(), 3, "All cards should be valid for first card");
        assert_eq!(valid, cards);
    }

    #[test]
    fn test_valid_plays_must_follow_suit() {
        let ace_hearts = Card::new(Suit::Hearts, Rank::Ace);
        let king_hearts = Card::new(Suit::Hearts, Rank::King);
        let queen_spades = Card::new(Suit::Spades, Rank::Queen);
        
        let cards = vec![ace_hearts, king_hearts, queen_spades];
        let hand = Hand::new(cards);
        
        let valid = hand.valid_plays(Some(Suit::Hearts));
        assert_eq!(valid.len(), 2, "Should only return hearts");
        assert!(valid.contains(&ace_hearts));
        assert!(valid.contains(&king_hearts));
        assert!(!valid.contains(&queen_spades));
    }

    #[test]
    fn test_valid_plays_cannot_follow_suit() {
        let queen_spades = Card::new(Suit::Spades, Rank::Queen);
        let jack_diamonds = Card::new(Suit::Diamonds, Rank::Jack);
        let ten_clubs = Card::new(Suit::Clubs, Rank::Ten);
        
        let cards = vec![queen_spades, jack_diamonds, ten_clubs];
        let hand = Hand::new(cards.clone());
        
        // Lead suit is Hearts, but hand has no hearts
        let valid = hand.valid_plays(Some(Suit::Hearts));
        assert_eq!(valid.len(), 3, "All cards should be valid when cannot follow suit");
        assert_eq!(valid, cards);
    }

    #[test]
    fn test_valid_plays_single_card_in_suit() {
        let ace_hearts = Card::new(Suit::Hearts, Rank::Ace);
        let queen_spades = Card::new(Suit::Spades, Rank::Queen);
        let jack_diamonds = Card::new(Suit::Diamonds, Rank::Jack);
        
        let cards = vec![ace_hearts, queen_spades, jack_diamonds];
        let hand = Hand::new(cards);
        
        let valid = hand.valid_plays(Some(Suit::Hearts));
        assert_eq!(valid.len(), 1, "Should only return the one heart");
        assert_eq!(valid[0], ace_hearts);
    }

    #[test]
    fn test_play_multiple_cards_in_sequence() {
        let ace_hearts = Card::new(Suit::Hearts, Rank::Ace);
        let king_spades = Card::new(Suit::Spades, Rank::King);
        let queen_diamonds = Card::new(Suit::Diamonds, Rank::Queen);
        
        let cards = vec![ace_hearts, king_spades, queen_diamonds];
        let mut hand = Hand::new(cards);
        
        assert_eq!(hand.cards().len(), 3);
        
        hand.play_card(ace_hearts).unwrap();
        assert_eq!(hand.cards().len(), 2);
        assert!(!hand.has_card(&ace_hearts));
        
        hand.play_card(king_spades).unwrap();
        assert_eq!(hand.cards().len(), 1);
        assert!(!hand.has_card(&king_spades));
        
        hand.play_card(queen_diamonds).unwrap();
        assert_eq!(hand.cards().len(), 0);
        assert!(!hand.has_card(&queen_diamonds));
    }
}
