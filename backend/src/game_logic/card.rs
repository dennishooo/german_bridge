use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rank {
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }

    pub fn value(&self, trump: Option<Suit>) -> u8 {
        // TODO: Implement card value calculation
        0
    }

    pub fn beats(&self, other: &Card, trump: Option<Suit>, lead_suit: Suit) -> bool {
        // TODO: Implement trick-taking logic
        false
    }
}
