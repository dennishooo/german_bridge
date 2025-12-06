use std::collections::HashMap;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use crate::connection::PlayerId;
use crate::game_logic::card::Suit;
use crate::game_logic::deck::{Deck, Hand};
use crate::game_logic::trick::{Trick, CompletedTrick};
use crate::game_logic::bidding::BiddingState;
use rand::seq::SliceRandom;
use tracing::{debug, info, warn};

pub struct GameState {
    pub phase: GamePhase,
    pub round_number: usize,
    pub cards_per_player: usize,
    pub deck: Deck,
    pub hands: HashMap<PlayerId, Hand>,
    pub current_trick: Trick,
    pub completed_tricks: Vec<CompletedTrick>,
    pub round_scores: HashMap<PlayerId, i32>,
    pub total_scores: HashMap<PlayerId, i32>,
    pub trump_suit: Option<Suit>,
    pub player_bids: HashMap<PlayerId, u8>,
    pub tricks_won: HashMap<PlayerId, u8>,
    pub current_player: PlayerId,
    pub first_bidder: PlayerId,
    pub turn_deadline: Option<Instant>,
    pub bidding_state: Option<BiddingState>,
    pub players: Vec<PlayerId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Bidding,
    Playing,
    RoundComplete,
    GameComplete,
}

impl GameState {
    /// Initialize a new game with players starting at round 1 with 1 card
    pub fn new(players: Vec<PlayerId>) -> Self {
        let num_players = players.len();
        let first_player = players[0];
        
        // Initialize empty collections
        let mut total_scores = HashMap::new();
        let mut tricks_won = HashMap::new();
        for player in &players {
            total_scores.insert(*player, 0);
            tricks_won.insert(*player, 0);
        }
        
        let mut state = Self {
            phase: GamePhase::Bidding,
            round_number: 1,
            cards_per_player: 1,
            deck: Deck::new_german_bridge(),
            hands: HashMap::new(),
            current_trick: Trick::new(),
            completed_tricks: Vec::new(),
            round_scores: HashMap::new(),
            total_scores,
            trump_suit: None,
            player_bids: HashMap::new(),
            tricks_won,
            current_player: first_player,
            first_bidder: first_player,
            turn_deadline: None,
            bidding_state: None,
            players,
        };
        
        // Start the first round
        state.start_round();
        state
    }
    
    /// Start a new round: deal cards, select random trump, reset round state
    pub fn start_round(&mut self) {
        // Create and shuffle a new deck
        self.deck = Deck::new_german_bridge();
        self.deck.shuffle();
        
        // Select random trump suit
        self.trump_suit = Some(Self::random_trump());
        
        // Deal cards to players
        let num_players = self.players.len();
        let total_cards = 52;
        
        // Calculate cards per player for this round
        // Start with 1 card in round 1, increment each round
        // Reset to 1 when we can't deal evenly anymore
        let max_cards_per_player = total_cards / num_players;
        if self.round_number > max_cards_per_player {
            self.round_number = 1;
            self.cards_per_player = 1;
        } else {
            self.cards_per_player = self.round_number;
        }
        
        info!("Starting round {} with {} cards per player, trump: {:?}", 
              self.round_number, self.cards_per_player, self.trump_suit);
        
        // Deal the cards
        let hands = self.deck.deal(num_players);
        self.hands.clear();
        for (i, hand) in hands.into_iter().enumerate() {
            self.hands.insert(self.players[i], hand);
        }
        
        // Reset round state
        self.phase = GamePhase::Bidding;
        self.current_trick = Trick::new();
        self.completed_tricks.clear();
        self.round_scores.clear();
        self.player_bids.clear();
        
        // Reset tricks won for this round
        for player in &self.players {
            self.tricks_won.insert(*player, 0);
        }
        
        // Set up bidding state
        self.current_player = self.first_bidder;
        self.bidding_state = Some(BiddingState::new(
            self.first_bidder,
            self.players.clone(),
            self.cards_per_player,
        ));
    }
    
    /// Select a random trump suit
    fn random_trump() -> Suit {
        let suits = [Suit::Clubs, Suit::Spades, Suit::Hearts, Suit::Diamonds];
        let mut rng = rand::thread_rng();
        *suits.choose(&mut rng).unwrap()
    }

    /// Validate a player action
    pub fn validate_action(&self, player_id: PlayerId, action: &crate::protocol::PlayerAction) -> Result<(), crate::error::GameError> {
        use crate::protocol::PlayerAction;
        
        // Check if it's the player's turn
        if player_id != self.current_player {
            return Err(crate::error::GameError::NotPlayerTurn);
        }
        
        // Check if player is in the game
        if !self.players.contains(&player_id) {
            return Err(crate::error::GameError::PlayerNotInGame);
        }
        
        match action {
            PlayerAction::Bid(bid) => {
                // Must be in bidding phase
                if self.phase != GamePhase::Bidding {
                    return Err(crate::error::GameError::InvalidMove(
                        "Not in bidding phase".to_string()
                    ));
                }
                
                // Validate the bid
                self.validate_bid(player_id, bid.tricks)?;
            }
            PlayerAction::PlayCard(card) => {
                // Must be in playing phase
                if self.phase != GamePhase::Playing {
                    return Err(crate::error::GameError::InvalidMove(
                        "Not in playing phase".to_string()
                    ));
                }
                
                // Check if player has the card
                let hand = self.hands.get(&player_id)
                    .ok_or_else(|| crate::error::GameError::PlayerNotInGame)?;
                
                if !hand.has_card(card) {
                    return Err(crate::error::GameError::InvalidMove(
                        "Card not in hand".to_string()
                    ));
                }
                
                // Check if card is a valid play (follows suit if required)
                let valid_plays = hand.valid_plays(self.current_trick.lead_suit);
                if !valid_plays.contains(card) {
                    return Err(crate::error::GameError::InvalidMove(
                        "Must follow suit if possible".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate a bid
    pub fn validate_bid(&self, player_id: PlayerId, bid: u8) -> Result<(), crate::error::GameError> {
        // Check bid range
        if bid as usize > self.cards_per_player {
            return Err(crate::error::GameError::InvalidMove(format!(
                "Bid {} exceeds cards dealt {}",
                bid, self.cards_per_player
            )));
        }
        
        // Check last bidder restriction
        if let Some(ref bidding_state) = self.bidding_state {
            if bidding_state.is_last_bidder(player_id) {
                bidding_state.validate_last_bid(bid)?;
            }
        }
        
        Ok(())
    }
    
    /// Apply a player action to update the game state
    pub fn apply_action(&mut self, player_id: PlayerId, action: crate::protocol::PlayerAction) -> Result<(), crate::error::GameError> {
        use crate::protocol::PlayerAction;
        
        // Validate the action first
        self.validate_action(player_id, &action)?;
        
        match action {
            PlayerAction::Bid(bid) => {
                // Record the bid
                self.player_bids.insert(player_id, bid.tricks);
                info!("Player {} bid {} tricks", player_id, bid.tricks);
                
                // Update bidding state
                if let Some(ref mut bidding_state) = self.bidding_state {
                    bidding_state.place_bid(player_id, bid.tricks)?;
                    
                    // Check if bidding is complete
                    if bidding_state.is_complete() {
                        // Transition to playing phase
                        self.phase = GamePhase::Playing;
                        self.current_player = self.first_bidder;
                        self.bidding_state = None;
                        info!("Bidding complete, transitioning to playing phase");
                    } else {
                        // Move to next bidder
                        self.current_player = bidding_state.current_bidder;
                        debug!("Next bidder: {}", self.current_player);
                    }
                }
            }
            PlayerAction::PlayCard(card) => {
                // Remove card from player's hand
                if let Some(hand) = self.hands.get_mut(&player_id) {
                    hand.play_card(card)?;
                }
                
                debug!("Player {} played card: {:?}", player_id, card);
                
                // Add card to current trick
                self.current_trick.add_card(player_id, card);
                
                // Check if trick is complete
                if self.current_trick.is_complete(self.players.len()) {
                    self.complete_trick()?;
                } else {
                    // Move to next player
                    self.advance_turn();
                }
            }
        }
        
        Ok(())
    }
    
    /// Complete a trick and update state
    fn complete_trick(&mut self) -> Result<(), crate::error::GameError> {
        // Determine the winner
        let winner = self.current_trick.winner(self.trump_suit)
            .ok_or_else(|| crate::error::GameError::InvalidMove(
                "Cannot determine trick winner".to_string()
            ))?;
        
        // Update tricks won
        *self.tricks_won.entry(winner).or_insert(0) += 1;
        
        info!("Trick won by player {} (total tricks: {})", winner, self.tricks_won[&winner]);
        
        // Store completed trick
        let completed = CompletedTrick {
            winner,
            cards: self.current_trick.cards.clone(),
            points: 0, // GBridge doesn't use points per trick
        };
        self.completed_tricks.push(completed);
        
        // Start new trick with winner leading
        self.current_trick = Trick::new();
        self.current_player = winner;
        
        // Check if round is complete (all cards played)
        let all_hands_empty = self.hands.values().all(|hand| hand.cards().is_empty());
        if all_hands_empty {
            self.calculate_round_scores();
            self.phase = GamePhase::RoundComplete;
            
            info!("Round {} complete. Scores: {:?}", self.round_number, self.round_scores);
            
            // Check if game should continue
            if self.should_continue_game() {
                // Advance to next round
                self.round_number += 1;
                // Rotate first bidder
                let current_index = self.players.iter()
                    .position(|p| *p == self.first_bidder)
                    .unwrap_or(0);
                let next_index = (current_index + 1) % self.players.len();
                self.first_bidder = self.players[next_index];
                
                self.start_round();
            } else {
                self.phase = GamePhase::GameComplete;
                info!("Game complete! Final scores: {:?}", self.total_scores);
            }
        }
        
        Ok(())
    }
    
    /// Calculate scores for the round using ScoreCalculator
    fn calculate_round_scores(&mut self) {
        use crate::game_logic::scoring::ScoreCalculator;
        use crate::game_logic::bidding::Bid;
        
        // Convert player_bids to HashMap<PlayerId, Bid>
        let bids: HashMap<PlayerId, Bid> = self.player_bids.iter()
            .map(|(player_id, tricks)| (*player_id, Bid { tricks: *tricks }))
            .collect();
        
        // Calculate round scores
        self.round_scores = ScoreCalculator::calculate_round_scores(&bids, &self.tricks_won);
        
        // Update total scores
        for (player_id, round_score) in &self.round_scores {
            *self.total_scores.entry(*player_id).or_insert(0) += round_score;
        }
    }
    
    /// Check if enough cards remain for the next round
    pub fn should_continue_game(&self) -> bool {
        let num_players = self.players.len();
        let total_cards = 52;
        let max_cards_per_player = total_cards / num_players;
        
        // Continue if we haven't reached the maximum cards per player yet
        self.round_number < max_cards_per_player
    }
    
    /// Advance to the next player in rotation
    pub fn advance_turn(&mut self) {
        let current_index = self.players.iter()
            .position(|p| *p == self.current_player)
            .unwrap_or(0);
        
        let next_index = (current_index + 1) % self.players.len();
        self.current_player = self.players[next_index];
    }
    
    /// Set the turn deadline for the current player
    pub fn set_turn_deadline(&mut self, timeout_secs: u64) {
        self.turn_deadline = Some(Instant::now() + std::time::Duration::from_secs(timeout_secs));
    }
    
    /// Check if the current turn has expired
    pub fn is_turn_expired(&self) -> bool {
        if let Some(deadline) = self.turn_deadline {
            Instant::now() >= deadline
        } else {
            false
        }
    }
    
    /// Get an automatic action for the current player on timeout
    pub fn get_auto_action(&self) -> Option<crate::protocol::PlayerAction> {
        use crate::protocol::PlayerAction;
        use crate::game_logic::bidding::Bid;
        
        match self.phase {
            GamePhase::Bidding => {
                // Auto-bid 0 (safest bid)
                warn!("Auto-bidding 0 for player {} due to timeout", self.current_player);
                Some(PlayerAction::Bid(Bid { tricks: 0 }))
            }
            GamePhase::Playing => {
                // Play the first valid card
                if let Some(hand) = self.hands.get(&self.current_player) {
                    let valid_plays = hand.valid_plays(self.current_trick.lead_suit);
                    if let Some(&card) = valid_plays.first() {
                        warn!("Auto-playing card {:?} for player {} due to timeout", card, self.current_player);
                        return Some(PlayerAction::PlayCard(card));
                    }
                }
                None
            }
            _ => None,
        }
    }
    
    /// Generate a player-specific view of the game state
    pub fn get_player_view(&self, player_id: PlayerId, game_id: crate::game::GameId) -> crate::protocol::PlayerGameView {
        use crate::protocol::PlayerGameView;
        
        // Get player's hand (or empty if not found)
        let your_hand = self.hands.get(&player_id)
            .map(|hand| hand.cards().to_vec())
            .unwrap_or_default();
        
        // Get current trick cards (visible to all)
        let current_trick = self.current_trick.cards.clone();
        
        // Get scores (use total scores)
        let scores = self.total_scores.clone();
        
        // Check if it's this player's turn
        let your_turn = self.current_player == player_id;
        
        PlayerGameView {
            game_id,
            phase: self.phase,
            your_hand,
            current_trick,
            scores,
            trump_suit: self.trump_suit,
            current_player: self.current_player,
            your_turn,
        }
    }
}
