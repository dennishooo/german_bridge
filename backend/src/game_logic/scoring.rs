use std::collections::HashMap;
use crate::connection::PlayerId;
use crate::game_logic::bidding::Bid;

pub struct ScoreCalculator;

impl ScoreCalculator {
    /// Calculate score for a player based on GBridge rules:
    /// - Made bid exactly: 10 + (tricks * tricks)
    /// - Missed bid: -((won - bid) * (won - bid))
    pub fn calculate_player_score(bid: u8, tricks_won: u8) -> i32 {
        if bid == tricks_won {
            // Made the bid exactly
            10 + (tricks_won as i32 * tricks_won as i32)
        } else {
            // Missed the bid
            let diff = (tricks_won as i32 - bid as i32).abs();
            -(diff * diff)
        }
    }

    /// Calculate scores for all players in a round
    pub fn calculate_round_scores(
        player_bids: &HashMap<PlayerId, Bid>,
        tricks_won: &HashMap<PlayerId, u8>,
    ) -> HashMap<PlayerId, i32> {
        player_bids
            .iter()
            .map(|(player_id, bid)| {
                let won = tricks_won.get(player_id).copied().unwrap_or(0);
                let score = Self::calculate_player_score(bid.tricks, won);
                (*player_id, score)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_made_bid() {
        // Made bid of 0: 10 + 0*0 = 10
        assert_eq!(ScoreCalculator::calculate_player_score(0, 0), 10);
        
        // Made bid of 1: 10 + 1*1 = 11
        assert_eq!(ScoreCalculator::calculate_player_score(1, 1), 11);
        
        // Made bid of 3: 10 + 3*3 = 19
        assert_eq!(ScoreCalculator::calculate_player_score(3, 3), 19);
        
        // Made bid of 5: 10 + 5*5 = 35
        assert_eq!(ScoreCalculator::calculate_player_score(5, 5), 35);
    }

    #[test]
    fn test_score_missed_bid() {
        // Bid 2, won 0: -(2-0)^2 = -4
        assert_eq!(ScoreCalculator::calculate_player_score(2, 0), -4);
        
        // Bid 2, won 1: -(2-1)^2 = -1
        assert_eq!(ScoreCalculator::calculate_player_score(2, 1), -1);
        
        // Bid 2, won 3: -(3-2)^2 = -1
        assert_eq!(ScoreCalculator::calculate_player_score(2, 3), -1);
        
        // Bid 2, won 5: -(5-2)^2 = -9
        assert_eq!(ScoreCalculator::calculate_player_score(2, 5), -9);
        
        // Bid 5, won 2: -(5-2)^2 = -9
        assert_eq!(ScoreCalculator::calculate_player_score(5, 2), -9);
    }
}
