use german_bridge_backend::protocol::{ClientMessage, ServerMessage};
use german_bridge_backend::connection::{ConnectionManager, PlayerId};
use tokio::sync::mpsc;
use axum::extract::ws::Message;
use serde_json;

#[tokio::test]
async fn test_player_connection_and_id_assignment() {
    // Create a connection manager
    let conn_manager = ConnectionManager::new();
    
    // Create a mock WebSocket sender
    let (tx, _rx) = mpsc::unbounded_channel();
    
    // Add a player
    let player_id = conn_manager.add_player(tx).await;
    
    // Verify player ID is assigned
    assert_ne!(player_id, PlayerId::nil());
    
    // Verify player is in active players list
    let active_players = conn_manager.get_active_players().await;
    assert_eq!(active_players.len(), 1);
    assert_eq!(active_players[0], player_id);
    
    // Verify stats
    let stats = conn_manager.get_stats().await;
    assert_eq!(stats.total_connections, 1);
    assert_eq!(stats.active_connections, 1);
    assert_eq!(stats.inactive_connections, 0);
}

#[tokio::test]
async fn test_message_serialization_deserialization() {
    // Test ClientMessage serialization
    let client_msg = ClientMessage::Ping;
    let json = serde_json::to_string(&client_msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ClientMessage::Ping => {},
        _ => panic!("Expected Ping message"),
    }
    
    // Test ServerMessage serialization
    let player_id = PlayerId::new_v4();
    let server_msg = ServerMessage::Connected { player_id };
    let json = serde_json::to_string(&server_msg).unwrap();
    let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        ServerMessage::Connected { player_id: id } => {
            assert_eq!(id, player_id);
        },
        _ => panic!("Expected Connected message"),
    }
}

#[tokio::test]
async fn test_send_to_player() {
    let conn_manager = ConnectionManager::new();
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    let player_id = conn_manager.add_player(tx).await;
    
    // Send a message to the player
    let msg = ServerMessage::Pong;
    conn_manager.send_to_player(player_id, msg).await;
    
    // Receive the message
    let received = rx.recv().await.unwrap();
    
    match received {
        Message::Text(text) => {
            let deserialized: ServerMessage = serde_json::from_str(&text).unwrap();
            match deserialized {
                ServerMessage::Pong => {},
                _ => panic!("Expected Pong message"),
            }
        },
        _ => panic!("Expected text message"),
    }
}

#[tokio::test]
async fn test_broadcast_to_players() {
    let conn_manager = ConnectionManager::new();
    
    // Create two players
    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let (tx2, mut rx2) = mpsc::unbounded_channel();
    
    let player1 = conn_manager.add_player(tx1).await;
    let player2 = conn_manager.add_player(tx2).await;
    
    // Broadcast a message
    let msg = ServerMessage::Pong;
    conn_manager.broadcast_to_players(&[player1, player2], msg).await;
    
    // Both players should receive the message
    let received1 = rx1.recv().await.unwrap();
    let received2 = rx2.recv().await.unwrap();
    
    for received in [received1, received2] {
        match received {
            Message::Text(text) => {
                let deserialized: ServerMessage = serde_json::from_str(&text).unwrap();
                match deserialized {
                    ServerMessage::Pong => {},
                    _ => panic!("Expected Pong message"),
                }
            },
            _ => panic!("Expected text message"),
        }
    }
}

#[tokio::test]
async fn test_player_disconnection() {
    let conn_manager = ConnectionManager::new();
    let (tx, _rx) = mpsc::unbounded_channel();
    
    let player_id = conn_manager.add_player(tx).await;
    
    // Verify player is active
    let active_players = conn_manager.get_active_players().await;
    assert_eq!(active_players.len(), 1);
    
    // Mark player as inactive
    let other_players = conn_manager.mark_inactive(player_id).await;
    assert_eq!(other_players.len(), 0); // No other players
    
    // Verify player is no longer active
    let active_players = conn_manager.get_active_players().await;
    assert_eq!(active_players.len(), 0);
    
    // Verify stats
    let stats = conn_manager.get_stats().await;
    assert_eq!(stats.total_connections, 1);
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.inactive_connections, 1);
}

#[tokio::test]
async fn test_player_reconnection() {
    let conn_manager = ConnectionManager::new();
    let (tx1, _rx1) = mpsc::unbounded_channel();
    
    let player_id = conn_manager.add_player(tx1).await;
    
    // Mark player as inactive
    conn_manager.mark_inactive(player_id).await;
    
    // Reconnect with new sender
    let (tx2, mut rx2) = mpsc::unbounded_channel();
    let result = conn_manager.reconnect_player(player_id, tx2).await;
    
    assert!(result.is_some());
    
    // Verify player is active again
    let active_players = conn_manager.get_active_players().await;
    assert_eq!(active_players.len(), 1);
    assert_eq!(active_players[0], player_id);
    
    // Verify we can send messages to reconnected player
    conn_manager.send_to_player(player_id, ServerMessage::Pong).await;
    let received = rx2.recv().await.unwrap();
    
    match received {
        Message::Text(text) => {
            let deserialized: ServerMessage = serde_json::from_str(&text).unwrap();
            match deserialized {
                ServerMessage::Pong => {},
                _ => panic!("Expected Pong message"),
            }
        },
        _ => panic!("Expected text message"),
    }
}

#[tokio::test]
async fn test_reconnection_timeout() {
    use std::time::Duration;
    
    // Create connection manager with very short timeout
    let conn_manager = ConnectionManager::with_reconnect_timeout(Duration::from_millis(100));
    let (tx1, _rx1) = mpsc::unbounded_channel();
    
    let player_id = conn_manager.add_player(tx1).await;
    
    // Mark player as inactive
    conn_manager.mark_inactive(player_id).await;
    
    // Wait for timeout to expire
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Try to reconnect - should fail
    let (tx2, _rx2) = mpsc::unbounded_channel();
    let result = conn_manager.reconnect_player(player_id, tx2).await;
    
    assert!(result.is_none());
}

#[tokio::test]
async fn test_cleanup_expired_sessions() {
    use std::time::Duration;
    
    // Create connection manager with very short timeout
    let conn_manager = ConnectionManager::with_reconnect_timeout(Duration::from_millis(100));
    let (tx, _rx) = mpsc::unbounded_channel();
    
    let player_id = conn_manager.add_player(tx).await;
    
    // Mark player as inactive
    conn_manager.mark_inactive(player_id).await;
    
    // Wait for timeout to expire
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Cleanup expired sessions
    let expired = conn_manager.cleanup_expired_sessions().await;
    
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0], player_id);
    
    // Verify player is removed
    let stats = conn_manager.get_stats().await;
    assert_eq!(stats.total_connections, 0);
}

#[tokio::test]
async fn test_multiple_players_disconnection_notification() {
    let conn_manager = ConnectionManager::new();
    
    // Create three players
    let (tx1, _rx1) = mpsc::unbounded_channel();
    let (tx2, _rx2) = mpsc::unbounded_channel();
    let (tx3, _rx3) = mpsc::unbounded_channel();
    
    let player1 = conn_manager.add_player(tx1).await;
    let player2 = conn_manager.add_player(tx2).await;
    let player3 = conn_manager.add_player(tx3).await;
    
    // Mark player2 as inactive
    let other_players = conn_manager.mark_inactive(player2).await;
    
    // Should return player1 and player3 as other active players
    assert_eq!(other_players.len(), 2);
    assert!(other_players.contains(&player1));
    assert!(other_players.contains(&player3));
}

// ============================================================================
// Lobby Flow Tests
// ============================================================================

use german_bridge_backend::lobby::LobbyManager;
use german_bridge_backend::game::GameManager;
use german_bridge_backend::protocol::{GameSettings, PlayerCount};
use german_bridge_backend::error::LobbyError;
use std::sync::Arc;

#[tokio::test]
async fn test_lobby_creation() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    let lobby_manager = LobbyManager::new(game_manager);
    
    let host_id = PlayerId::new_v4();
    let settings = GameSettings {
        player_count: PlayerCount::Four,
        turn_timeout_secs: 30,
        allow_reconnect: true,
    };
    
    let lobby_id = lobby_manager.create_lobby(host_id, settings).await;
    
    // Verify lobby was created
    let lobby = lobby_manager.get_lobby(lobby_id).await;
    assert!(lobby.is_some());
    
    let lobby = lobby.unwrap();
    assert_eq!(lobby.host, host_id);
    assert_eq!(lobby.players.len(), 1);
    assert_eq!(lobby.players[0], host_id);
    assert_eq!(lobby.max_players, 4);
}

#[tokio::test]
async fn test_lobby_joining() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    let lobby_manager = LobbyManager::new(game_manager);
    
    let host_id = PlayerId::new_v4();
    let player2_id = PlayerId::new_v4();
    let settings = GameSettings::default();
    
    let lobby_id = lobby_manager.create_lobby(host_id, settings).await;
    
    // Join the lobby
    let result = lobby_manager.join_lobby(lobby_id, player2_id).await;
    assert!(result.is_ok());
    
    // Verify player was added
    let lobby = lobby_manager.get_lobby(lobby_id).await.unwrap();
    assert_eq!(lobby.players.len(), 2);
    assert!(lobby.players.contains(&host_id));
    assert!(lobby.players.contains(&player2_id));
}

#[tokio::test]
async fn test_lobby_full_rejection() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    let lobby_manager = LobbyManager::new(game_manager);
    
    let host_id = PlayerId::new_v4();
    let settings = GameSettings {
        player_count: PlayerCount::Three,
        turn_timeout_secs: 30,
        allow_reconnect: true,
    };
    
    let lobby_id = lobby_manager.create_lobby(host_id, settings).await;
    
    // Add two more players to fill the lobby (3 total)
    let player2_id = PlayerId::new_v4();
    let player3_id = PlayerId::new_v4();
    lobby_manager.join_lobby(lobby_id, player2_id).await.unwrap();
    lobby_manager.join_lobby(lobby_id, player3_id).await.unwrap();
    
    // Try to add a fourth player - should fail
    let player4_id = PlayerId::new_v4();
    let result = lobby_manager.join_lobby(lobby_id, player4_id).await;
    
    assert!(result.is_err());
    match result {
        Err(LobbyError::LobbyFull) => {},
        _ => panic!("Expected LobbyFull error"),
    }
}

#[tokio::test]
async fn test_lobby_host_transfer_on_leave() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    let lobby_manager = LobbyManager::new(game_manager);
    
    let host_id = PlayerId::new_v4();
    let player2_id = PlayerId::new_v4();
    let settings = GameSettings::default();
    
    let lobby_id = lobby_manager.create_lobby(host_id, settings).await;
    lobby_manager.join_lobby(lobby_id, player2_id).await.unwrap();
    
    // Host leaves
    lobby_manager.leave_lobby(lobby_id, host_id).await.unwrap();
    
    // Verify host was transferred to player2
    let lobby = lobby_manager.get_lobby(lobby_id).await.unwrap();
    assert_eq!(lobby.host, player2_id);
    assert_eq!(lobby.players.len(), 1);
    assert_eq!(lobby.players[0], player2_id);
}

#[tokio::test]
async fn test_lobby_removed_when_empty() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    let lobby_manager = LobbyManager::new(game_manager);
    
    let host_id = PlayerId::new_v4();
    let settings = GameSettings::default();
    
    let lobby_id = lobby_manager.create_lobby(host_id, settings).await;
    
    // Host leaves (only player)
    lobby_manager.leave_lobby(lobby_id, host_id).await.unwrap();
    
    // Verify lobby was removed
    let lobby = lobby_manager.get_lobby(lobby_id).await;
    assert!(lobby.is_none());
}

#[tokio::test]
async fn test_list_lobbies() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    let lobby_manager = LobbyManager::new(game_manager);
    
    let host1_id = PlayerId::new_v4();
    let host2_id = PlayerId::new_v4();
    let settings = GameSettings::default();
    
    // Create two lobbies
    let lobby1_id = lobby_manager.create_lobby(host1_id, settings.clone()).await;
    let lobby2_id = lobby_manager.create_lobby(host2_id, settings).await;
    
    // List lobbies
    let lobbies = lobby_manager.list_lobbies().await;
    
    assert_eq!(lobbies.len(), 2);
    let lobby_ids: Vec<_> = lobbies.iter().map(|l| l.id).collect();
    assert!(lobby_ids.contains(&lobby1_id));
    assert!(lobby_ids.contains(&lobby2_id));
}

#[tokio::test]
async fn test_list_lobbies_excludes_full() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    let lobby_manager = LobbyManager::new(game_manager);
    
    let host_id = PlayerId::new_v4();
    let settings = GameSettings {
        player_count: PlayerCount::Three,
        turn_timeout_secs: 30,
        allow_reconnect: true,
    };
    
    let lobby_id = lobby_manager.create_lobby(host_id, settings).await;
    
    // Fill the lobby
    let player2_id = PlayerId::new_v4();
    let player3_id = PlayerId::new_v4();
    lobby_manager.join_lobby(lobby_id, player2_id).await.unwrap();
    lobby_manager.join_lobby(lobby_id, player3_id).await.unwrap();
    
    // List lobbies - should be empty since the only lobby is full
    let lobbies = lobby_manager.list_lobbies().await;
    assert_eq!(lobbies.len(), 0);
}

#[tokio::test]
async fn test_game_start_with_correct_player_count() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    let lobby_manager = LobbyManager::new(game_manager);
    
    let host_id = PlayerId::new_v4();
    let player2_id = PlayerId::new_v4();
    let settings = GameSettings::default();
    
    let lobby_id = lobby_manager.create_lobby(host_id, settings).await;
    lobby_manager.join_lobby(lobby_id, player2_id).await.unwrap();
    
    // Start game
    let result = lobby_manager.start_game(lobby_id, host_id).await;
    assert!(result.is_ok());
    
    // Verify lobby was removed after game start
    let lobby = lobby_manager.get_lobby(lobby_id).await;
    assert!(lobby.is_none());
}

#[tokio::test]
async fn test_game_start_requires_host() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    let lobby_manager = LobbyManager::new(game_manager);
    
    let host_id = PlayerId::new_v4();
    let player2_id = PlayerId::new_v4();
    let settings = GameSettings::default();
    
    let lobby_id = lobby_manager.create_lobby(host_id, settings).await;
    lobby_manager.join_lobby(lobby_id, player2_id).await.unwrap();
    
    // Try to start game as non-host
    let result = lobby_manager.start_game(lobby_id, player2_id).await;
    
    assert!(result.is_err());
    match result {
        Err(LobbyError::NotHost) => {},
        _ => panic!("Expected NotHost error"),
    }
}

#[tokio::test]
async fn test_game_start_requires_minimum_players() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    let lobby_manager = LobbyManager::new(game_manager);
    
    let host_id = PlayerId::new_v4();
    let settings = GameSettings::default();
    
    let lobby_id = lobby_manager.create_lobby(host_id, settings).await;
    
    // Try to start game with only 1 player
    let result = lobby_manager.start_game(lobby_id, host_id).await;
    
    assert!(result.is_err());
    match result {
        Err(LobbyError::NotEnoughPlayers) => {},
        _ => panic!("Expected NotEnoughPlayers error"),
    }
}

// ============================================================================
// Complete Game Flow Tests
// ============================================================================

use german_bridge_backend::game_state::{GameState, GamePhase};
use german_bridge_backend::game_logic::card::{Card, Suit, Rank};
use german_bridge_backend::game_logic::bidding::Bid;
use german_bridge_backend::protocol::PlayerAction;
use german_bridge_backend::error::GameError;

#[tokio::test]
async fn test_game_initialization_and_card_dealing() {
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let players = vec![player1, player2];
    
    let game_state = GameState::new(players.clone());
    
    // Verify initial state
    assert_eq!(game_state.phase, GamePhase::Bidding);
    assert_eq!(game_state.round_number, 1);
    assert_eq!(game_state.cards_per_player, 1);
    assert!(game_state.trump_suit.is_some());
    
    // Verify each player has cards
    assert_eq!(game_state.hands.len(), 2);
    for player in &players {
        let hand = game_state.hands.get(player).unwrap();
        assert_eq!(hand.cards().len(), 1); // Round 1 = 1 card per player
    }
    
    // Verify bidding state is initialized
    assert!(game_state.bidding_state.is_some());
}

#[tokio::test]
async fn test_bidding_phase_completion() {
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let players = vec![player1, player2];
    
    let mut game_state = GameState::new(players.clone());
    
    // Player 1 bids
    let bid1 = PlayerAction::Bid(Bid { tricks: 1 });
    game_state.apply_action(player1, bid1).unwrap();
    
    // Should still be in bidding phase
    assert_eq!(game_state.phase, GamePhase::Bidding);
    
    // Player 2 bids (last bidder, cannot bid 0 since 1+0=1 which equals cards)
    let bid2 = PlayerAction::Bid(Bid { tricks: 1 });
    game_state.apply_action(player2, bid2).unwrap();
    
    // Should transition to playing phase
    assert_eq!(game_state.phase, GamePhase::Playing);
    assert!(game_state.bidding_state.is_none());
}

#[tokio::test]
async fn test_last_bidder_restriction() {
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let players = vec![player1, player2];
    
    let mut game_state = GameState::new(players.clone());
    
    // Player 1 bids 1
    let bid1 = PlayerAction::Bid(Bid { tricks: 1 });
    game_state.apply_action(player1, bid1).unwrap();
    
    // Player 2 tries to bid 0 (sum would be 1, which equals cards_per_player)
    let bid2 = PlayerAction::Bid(Bid { tricks: 0 });
    let result = game_state.apply_action(player2, bid2);
    
    assert!(result.is_err());
    match result {
        Err(GameError::InvalidMove(_)) => {},
        _ => panic!("Expected InvalidMove error"),
    }
}

#[tokio::test]
async fn test_playing_phase_with_valid_moves() {
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let players = vec![player1, player2];
    
    let mut game_state = GameState::new(players.clone());
    
    // Complete bidding
    game_state.apply_action(player1, PlayerAction::Bid(Bid { tricks: 1 })).unwrap();
    game_state.apply_action(player2, PlayerAction::Bid(Bid { tricks: 1 })).unwrap();
    
    assert_eq!(game_state.phase, GamePhase::Playing);
    
    // Get the cards from each player's hand
    let player1_card = game_state.hands.get(&player1).unwrap().cards()[0];
    let player2_card = game_state.hands.get(&player2).unwrap().cards()[0];
    
    // Player 1 plays their card
    game_state.apply_action(player1, PlayerAction::PlayCard(player1_card)).unwrap();
    
    // Player 2 plays their card
    game_state.apply_action(player2, PlayerAction::PlayCard(player2_card)).unwrap();
    
    // Round should be complete after both players play their only card
    assert!(game_state.phase == GamePhase::RoundComplete || game_state.phase == GamePhase::Bidding);
}

#[tokio::test]
async fn test_invalid_card_play_not_in_hand() {
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let players = vec![player1, player2];
    
    let mut game_state = GameState::new(players.clone());
    
    // Complete bidding (player1 bids 1, player2 cannot bid 0 due to last bidder rule)
    game_state.apply_action(player1, PlayerAction::Bid(Bid { tricks: 1 })).unwrap();
    game_state.apply_action(player2, PlayerAction::Bid(Bid { tricks: 1 })).unwrap();
    
    // Try to play a card that's not in hand
    let fake_card = Card::new(Suit::Hearts, Rank::Ace);
    let result = game_state.apply_action(player1, PlayerAction::PlayCard(fake_card));
    
    assert!(result.is_err());
    match result {
        Err(GameError::InvalidMove(_)) => {},
        _ => panic!("Expected InvalidMove error"),
    }
}

#[tokio::test]
async fn test_not_player_turn_error() {
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let players = vec![player1, player2];
    
    let mut game_state = GameState::new(players.clone());
    
    // Try to have player 2 bid when it's player 1's turn
    let bid = PlayerAction::Bid(Bid { tricks: 0 });
    let result = game_state.apply_action(player2, bid);
    
    assert!(result.is_err());
    match result {
        Err(GameError::NotPlayerTurn) => {},
        _ => panic!("Expected NotPlayerTurn error"),
    }
}

#[tokio::test]
async fn test_game_completion_and_scoring() {
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let players = vec![player1, player2];
    
    let mut game_state = GameState::new(players.clone());
    
    // Complete bidding
    game_state.apply_action(player1, PlayerAction::Bid(Bid { tricks: 1 })).unwrap();
    game_state.apply_action(player2, PlayerAction::Bid(Bid { tricks: 1 })).unwrap();
    
    // Play cards
    let player1_card = game_state.hands.get(&player1).unwrap().cards()[0];
    let player2_card = game_state.hands.get(&player2).unwrap().cards()[0];
    
    game_state.apply_action(player1, PlayerAction::PlayCard(player1_card)).unwrap();
    game_state.apply_action(player2, PlayerAction::PlayCard(player2_card)).unwrap();
    
    // After round completes, scores should be calculated
    // The game may have moved to next round or completed
    // Check that total scores exist (they're initialized at game start)
    assert!(!game_state.total_scores.is_empty());
    assert_eq!(game_state.total_scores.len(), 2);
    
    // Verify scores were updated (at least one should be non-zero after the round)
    let player1_total = game_state.total_scores.get(&player1).unwrap();
    let player2_total = game_state.total_scores.get(&player2).unwrap();
    
    // One player won the trick, so scores should have changed
    // One should be positive (made bid), one should be negative (missed bid)
    assert!(*player1_total != 0 || *player2_total != 0, "At least one score should be non-zero");
}

#[tokio::test]
async fn test_round_progression() {
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let players = vec![player1, player2];
    
    let mut game_state = GameState::new(players.clone());
    
    assert_eq!(game_state.round_number, 1);
    assert_eq!(game_state.cards_per_player, 1);
    
    // Complete round 1 (player1 bids 1, player2 cannot bid 0 due to last bidder rule, so bids 1)
    game_state.apply_action(player1, PlayerAction::Bid(Bid { tricks: 1 })).unwrap();
    game_state.apply_action(player2, PlayerAction::Bid(Bid { tricks: 1 })).unwrap();
    
    let player1_card = game_state.hands.get(&player1).unwrap().cards()[0];
    let player2_card = game_state.hands.get(&player2).unwrap().cards()[0];
    
    game_state.apply_action(player1, PlayerAction::PlayCard(player1_card)).unwrap();
    game_state.apply_action(player2, PlayerAction::PlayCard(player2_card)).unwrap();
    
    // Should progress to round 2 if game continues
    if game_state.phase == GamePhase::Bidding {
        assert_eq!(game_state.round_number, 2);
        assert_eq!(game_state.cards_per_player, 2);
    }
}

#[tokio::test]
async fn test_three_player_game() {
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let player3 = PlayerId::new_v4();
    let players = vec![player1, player2, player3];
    
    let game_state = GameState::new(players.clone());
    
    // Verify all three players have cards
    assert_eq!(game_state.hands.len(), 3);
    for player in &players {
        let hand = game_state.hands.get(player).unwrap();
        assert_eq!(hand.cards().len(), 1);
    }
}

#[tokio::test]
async fn test_game_manager_integration() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let players = vec![player1, player2];
    
    // Create a game
    let game_id = game_manager.create_game(players.clone()).await;
    
    // Get game state for player 1
    let view = game_manager.get_game_state(game_id, player1).await;
    assert!(view.is_ok());
    
    let view = view.unwrap();
    assert_eq!(view.game_id, game_id);
    assert_eq!(view.phase, GamePhase::Bidding);
    assert_eq!(view.your_hand.len(), 1);
    
    // Place a bid
    let result = game_manager.handle_player_action(
        game_id,
        player1,
        PlayerAction::Bid(Bid { tricks: 0 })
    ).await;
    assert!(result.is_ok());
    
    // Get updated state
    let view = game_manager.get_game_state(game_id, player1).await.unwrap();
    assert_eq!(view.phase, GamePhase::Bidding); // Still bidding, waiting for player 2
}

#[tokio::test]
async fn test_player_not_in_game_error() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(conn_manager));
    
    let player1 = PlayerId::new_v4();
    let player2 = PlayerId::new_v4();
    let player3 = PlayerId::new_v4(); // Not in game
    let players = vec![player1, player2];
    
    let game_id = game_manager.create_game(players).await;
    
    // Try to get state for player not in game
    let result = game_manager.get_game_state(game_id, player3).await;
    assert!(result.is_err());
    match result {
        Err(GameError::PlayerNotInGame) => {},
        _ => panic!("Expected PlayerNotInGame error"),
    }
    
    // Try to perform action as player not in game
    let result = game_manager.handle_player_action(
        game_id,
        player3,
        PlayerAction::Bid(Bid { tricks: 0 })
    ).await;
    assert!(result.is_err());
}

// ============================================================================
// Reconnection Scenario Tests
// ============================================================================

#[tokio::test]
async fn test_player_disconnect_during_game() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(Arc::clone(&conn_manager)));
    
    let (tx1, _rx1) = mpsc::unbounded_channel();
    let (tx2, _rx2) = mpsc::unbounded_channel();
    
    let player1 = conn_manager.add_player(tx1).await;
    let player2 = conn_manager.add_player(tx2).await;
    
    // Create a game
    let game_id = game_manager.create_game(vec![player1, player2]).await;
    
    // Verify both players are active
    let active_players = conn_manager.get_active_players().await;
    assert_eq!(active_players.len(), 2);
    
    // Player 1 disconnects
    let other_players = conn_manager.mark_inactive(player1).await;
    assert_eq!(other_players.len(), 1);
    assert_eq!(other_players[0], player2);
    
    // Verify player 1 is inactive
    let active_players = conn_manager.get_active_players().await;
    assert_eq!(active_players.len(), 1);
    assert_eq!(active_players[0], player2);
    
    // Game should still exist
    let result = game_manager.get_game_state(game_id, player2).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_successful_reconnection_and_state_restoration() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(Arc::clone(&conn_manager)));
    
    let (tx1, _rx1) = mpsc::unbounded_channel();
    let (tx2, _rx2) = mpsc::unbounded_channel();
    
    let player1 = conn_manager.add_player(tx1).await;
    let player2 = conn_manager.add_player(tx2).await;
    
    // Create a game
    let game_id = game_manager.create_game(vec![player1, player2]).await;
    
    // Player 1 makes a bid
    game_manager.handle_player_action(
        game_id,
        player1,
        PlayerAction::Bid(Bid { tricks: 1 })
    ).await.unwrap();
    
    // Player 1 disconnects
    conn_manager.mark_inactive(player1).await;
    
    // Player 1 reconnects
    let (tx1_new, mut rx1_new) = mpsc::unbounded_channel();
    let result = conn_manager.reconnect_player(player1, tx1_new).await;
    assert!(result.is_some());
    
    // Verify player 1 is active again
    let active_players = conn_manager.get_active_players().await;
    assert_eq!(active_players.len(), 2);
    
    // Player 1 should be able to get their game state
    let view = game_manager.get_game_state(game_id, player1).await;
    assert!(view.is_ok());
    
    let view = view.unwrap();
    assert_eq!(view.game_id, game_id);
    assert_eq!(view.your_hand.len(), 1);
    
    // Verify we can send messages to reconnected player
    conn_manager.send_to_player(player1, ServerMessage::Pong).await;
    let received = rx1_new.recv().await;
    assert!(received.is_some());
}

#[tokio::test]
async fn test_reconnection_timeout_and_player_removal() {
    use std::time::Duration;
    
    // Create connection manager with very short timeout
    let conn_manager = Arc::new(ConnectionManager::with_reconnect_timeout(Duration::from_millis(100)));
    let game_manager = Arc::new(GameManager::new(Arc::clone(&conn_manager)));
    
    let (tx1, _rx1) = mpsc::unbounded_channel();
    let (tx2, _rx2) = mpsc::unbounded_channel();
    
    let player1 = conn_manager.add_player(tx1).await;
    let player2 = conn_manager.add_player(tx2).await;
    
    // Create a game
    let _game_id = game_manager.create_game(vec![player1, player2]).await;
    
    // Player 1 disconnects
    conn_manager.mark_inactive(player1).await;
    
    // Wait for timeout to expire
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Try to reconnect - should fail
    let (tx1_new, _rx1_new) = mpsc::unbounded_channel();
    let result = conn_manager.reconnect_player(player1, tx1_new).await;
    assert!(result.is_none());
    
    // Cleanup expired sessions
    let expired = conn_manager.cleanup_expired_sessions().await;
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0], player1);
}

#[tokio::test]
async fn test_multiple_disconnects_and_reconnects() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(Arc::clone(&conn_manager)));
    
    let (tx1, _rx1) = mpsc::unbounded_channel();
    let (tx2, _rx2) = mpsc::unbounded_channel();
    let (tx3, _rx3) = mpsc::unbounded_channel();
    
    let player1 = conn_manager.add_player(tx1).await;
    let player2 = conn_manager.add_player(tx2).await;
    let player3 = conn_manager.add_player(tx3).await;
    
    // Create a game
    let game_id = game_manager.create_game(vec![player1, player2, player3]).await;
    
    // Player 1 and 3 disconnect
    conn_manager.mark_inactive(player1).await;
    conn_manager.mark_inactive(player3).await;
    
    // Only player 2 should be active
    let active_players = conn_manager.get_active_players().await;
    assert_eq!(active_players.len(), 1);
    assert_eq!(active_players[0], player2);
    
    // Player 1 reconnects
    let (tx1_new, _rx1_new) = mpsc::unbounded_channel();
    let result = conn_manager.reconnect_player(player1, tx1_new).await;
    assert!(result.is_some());
    
    // Now player 1 and 2 should be active
    let active_players = conn_manager.get_active_players().await;
    assert_eq!(active_players.len(), 2);
    assert!(active_players.contains(&player1));
    assert!(active_players.contains(&player2));
    
    // Player 3 reconnects
    let (tx3_new, _rx3_new) = mpsc::unbounded_channel();
    let result = conn_manager.reconnect_player(player3, tx3_new).await;
    assert!(result.is_some());
    
    // All players should be active
    let active_players = conn_manager.get_active_players().await;
    assert_eq!(active_players.len(), 3);
    
    // Game should still be accessible
    let view = game_manager.get_game_state(game_id, player1).await;
    assert!(view.is_ok());
}

#[tokio::test]
async fn test_disconnect_during_bidding_phase() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(Arc::clone(&conn_manager)));
    
    let (tx1, _rx1) = mpsc::unbounded_channel();
    let (tx2, _rx2) = mpsc::unbounded_channel();
    
    let player1 = conn_manager.add_player(tx1).await;
    let player2 = conn_manager.add_player(tx2).await;
    
    // Create a game
    let game_id = game_manager.create_game(vec![player1, player2]).await;
    
    // Player 1 bids
    game_manager.handle_player_action(
        game_id,
        player1,
        PlayerAction::Bid(Bid { tricks: 1 })
    ).await.unwrap();
    
    // Player 2 disconnects before bidding
    conn_manager.mark_inactive(player2).await;
    
    // Player 2 reconnects
    let (tx2_new, _rx2_new) = mpsc::unbounded_channel();
    conn_manager.reconnect_player(player2, tx2_new).await;
    
    // Player 2 should be able to continue and place their bid
    let result = game_manager.handle_player_action(
        game_id,
        player2,
        PlayerAction::Bid(Bid { tricks: 1 })
    ).await;
    assert!(result.is_ok());
    
    // Game should have progressed to playing phase
    let view = game_manager.get_game_state(game_id, player1).await.unwrap();
    assert_eq!(view.phase, GamePhase::Playing);
}

#[tokio::test]
async fn test_disconnect_during_playing_phase() {
    let conn_manager = Arc::new(ConnectionManager::new());
    let game_manager = Arc::new(GameManager::new(Arc::clone(&conn_manager)));
    
    let (tx1, _rx1) = mpsc::unbounded_channel();
    let (tx2, _rx2) = mpsc::unbounded_channel();
    
    let player1 = conn_manager.add_player(tx1).await;
    let player2 = conn_manager.add_player(tx2).await;
    
    // Create a game and complete bidding
    let game_id = game_manager.create_game(vec![player1, player2]).await;
    
    game_manager.handle_player_action(
        game_id,
        player1,
        PlayerAction::Bid(Bid { tricks: 1 })
    ).await.unwrap();
    
    game_manager.handle_player_action(
        game_id,
        player2,
        PlayerAction::Bid(Bid { tricks: 1 })
    ).await.unwrap();
    
    // Get player 1's card
    let view = game_manager.get_game_state(game_id, player1).await.unwrap();
    let player1_card = view.your_hand[0];
    
    // Player 1 plays a card
    game_manager.handle_player_action(
        game_id,
        player1,
        PlayerAction::PlayCard(player1_card)
    ).await.unwrap();
    
    // Player 2 disconnects before playing
    conn_manager.mark_inactive(player2).await;
    
    // Player 2 reconnects
    let (tx2_new, _rx2_new) = mpsc::unbounded_channel();
    conn_manager.reconnect_player(player2, tx2_new).await;
    
    // Player 2 should be able to get their state and play
    let view = game_manager.get_game_state(game_id, player2).await.unwrap();
    assert_eq!(view.phase, GamePhase::Playing);
    assert_eq!(view.your_hand.len(), 1);
    
    let player2_card = view.your_hand[0];
    let result = game_manager.handle_player_action(
        game_id,
        player2,
        PlayerAction::PlayCard(player2_card)
    ).await;
    assert!(result.is_ok());
}
