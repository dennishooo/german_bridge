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
    Two,
    Three,
    Four,
    Five,
    Six,
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

    /// GBridge doesn't use point values - scoring is based on tricks won vs bid
    /// This method is kept for potential future use but returns 0
    pub fn value(&self, _trump: Option<Suit>) -> u8 {
        0
    }

    /// Determines if this card beats another card in a trick
    /// Trump cards beat non-trump cards
    /// Within the same suit, higher rank wins
    /// Cards not following lead suit cannot win (unless trump)
    pub fn beats(&self, other: &Card, trump: Option<Suit>, lead_suit: Suit) -> bool {
        let self_is_trump = trump.map_or(false, |t| self.suit == t);
        let other_is_trump = trump.map_or(false, |t| other.suit == t);

        // Trump always beats non-trump
        if self_is_trump && !other_is_trump {
            return true;
        }
        if !self_is_trump && other_is_trump {
            return false;
        }

        // Both trump or both non-trump
        if self.suit == other.suit {
            // Same suit: compare ranks
            self.rank > other.rank
        } else {
            // Different suits: only lead suit can win
            self.suit == lead_suit
        }
    }
}

impl PartialOrd for Rank {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rank {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let rank_value = |r: &Rank| match r {
            Rank::Two => 0,
            Rank::Three => 1,
            Rank::Four => 2,
            Rank::Five => 3,
            Rank::Six => 4,
            Rank::Seven => 5,
            Rank::Eight => 6,
            Rank::Nine => 7,
            Rank::Ten => 8,
            Rank::Jack => 9,
            Rank::Queen => 10,
            Rank::King => 11,
            Rank::Ace => 12,
        };
        rank_value(self).cmp(&rank_value(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_values() {
        // GBridge doesn't use point values
        let ace = Card::new(Suit::Hearts, Rank::Ace);
        let two = Card::new(Suit::Hearts, Rank::Two);
        
        assert_eq!(ace.value(None), 0);
        assert_eq!(two.value(None), 0);
    }

    #[test]
    fn test_rank_ordering() {
        // Standard poker ranking: A > K > Q > J > 10 > 9 > 8 > 7 > 6 > 5 > 4 > 3 > 2
        assert!(Rank::Ace > Rank::King);
        assert!(Rank::King > Rank::Queen);
        assert!(Rank::Queen > Rank::Jack);
        assert!(Rank::Jack > Rank::Ten);
        assert!(Rank::Ten > Rank::Nine);
        assert!(Rank::Nine > Rank::Eight);
        assert!(Rank::Eight > Rank::Seven);
        assert!(Rank::Seven > Rank::Six);
        assert!(Rank::Six > Rank::Five);
        assert!(Rank::Five > Rank::Four);
        assert!(Rank::Four > Rank::Three);
        assert!(Rank::Three > Rank::Two);
    }

    #[test]
    fn test_beats_same_suit() {
        let ace_hearts = Card::new(Suit::Hearts, Rank::Ace);
        let king_hearts = Card::new(Suit::Hearts, Rank::King);
        let ten_hearts = Card::new(Suit::Hearts, Rank::Ten);
        let two_hearts = Card::new(Suit::Hearts, Rank::Two);

        // Higher rank beats lower rank in same suit (A > K > 10 > 2)
        assert!(ace_hearts.beats(&king_hearts, None, Suit::Hearts));
        assert!(king_hearts.beats(&ten_hearts, None, Suit::Hearts));
        assert!(ten_hearts.beats(&two_hearts, None, Suit::Hearts));
        assert!(!two_hearts.beats(&ace_hearts, None, Suit::Hearts));
    }

    #[test]
    fn test_beats_trump() {
        let seven_clubs = Card::new(Suit::Clubs, Rank::Seven);
        let ace_hearts = Card::new(Suit::Hearts, Rank::Ace);

        // Trump beats non-trump regardless of rank
        assert!(seven_clubs.beats(&ace_hearts, Some(Suit::Clubs), Suit::Hearts));
        assert!(!ace_hearts.beats(&seven_clubs, Some(Suit::Clubs), Suit::Hearts));
    }

    #[test]
    fn test_beats_lead_suit() {
        let ace_hearts = Card::new(Suit::Hearts, Rank::Ace);
        let ace_spades = Card::new(Suit::Spades, Rank::Ace);
        let seven_hearts = Card::new(Suit::Hearts, Rank::Seven);

        // Lead suit beats non-lead suit (no trump)
        assert!(seven_hearts.beats(&ace_spades, None, Suit::Hearts));
        assert!(!ace_spades.beats(&seven_hearts, None, Suit::Hearts));
        
        // Both following lead suit: higher rank wins
        assert!(ace_hearts.beats(&seven_hearts, None, Suit::Hearts));
    }

    #[test]
    fn test_beats_trump_vs_lead() {
        let seven_clubs = Card::new(Suit::Clubs, Rank::Seven);
        let ace_hearts = Card::new(Suit::Hearts, Rank::Ace);

        // Trump beats lead suit
        assert!(seven_clubs.beats(&ace_hearts, Some(Suit::Clubs), Suit::Hearts));
    }

    #[test]
    fn test_beats_both_trump() {
        let ace_clubs = Card::new(Suit::Clubs, Rank::Ace);
        let seven_clubs = Card::new(Suit::Clubs, Rank::Seven);

        // Both trump: higher rank wins
        assert!(ace_clubs.beats(&seven_clubs, Some(Suit::Clubs), Suit::Hearts));
        assert!(!seven_clubs.beats(&ace_clubs, Some(Suit::Clubs), Suit::Hearts));
    }
}
