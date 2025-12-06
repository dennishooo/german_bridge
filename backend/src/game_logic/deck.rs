use crate::game_logic::card::{Card, Suit, Rank};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    /// Create a standard 52-card deck for GBridge
    pub fn new_standard() -> Self {
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

    pub fn deal(&mut self, num_cards: usize, num_players: usize) -> Vec<Hand> {
        let mut hands = vec![Vec::new(); num_players];
        
        for _ in 0..num_cards {
            for player_idx in 0..num_players {
                if let Some(card) = self.cards.pop() {
                    hands[player_idx].push(card);
                }
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

    pub fn play_card(&mut self, card: Card) -> Result<Card, crate::error::GameError> {
        // TODO: Implement card playing
        Ok(card)
    }

    pub fn valid_plays(&self, lead_suit: Option<Suit>) -> Vec<Card> {
        // TODO: Implement valid plays logic
        Vec::new()
    }
}
