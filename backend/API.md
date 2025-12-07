# GBridge Backend API Documentation

## Overview

The GBridge backend uses WebSocket connections for real-time bidirectional communication. All messages are sent as JSON with a `type` field indicating the message type and an optional `payload` field containing the message data.

## Connection

### Authentication

Before connecting to the WebSocket, you must first authenticate via HTTP to receive a JWT token.

#### Register

**Endpoint:** `POST /api/register`

**Request:**

```json
{
  "username": "player1",
  "password": "secret123"
}
```

**Response:**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "username": "player1",
  "user_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### Login

**Endpoint:** `POST /api/login`

**Request:**

```json
{
  "username": "player1",
  "password": "secret123"
}
```

**Response:**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "username": "player1",
  "user_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### WebSocket Endpoint

```
ws://localhost:8080/ws?token=<JWT_TOKEN>
```

**Note:** The JWT token from login/register must be included as a query parameter.

### Connection Flow

1. Client registers or logs in via HTTP to receive JWT token
2. Client connects to WebSocket endpoint with token: `ws://localhost:8080/ws?token=<JWT>`
3. Server validates JWT and sends `Connected` message with player ID
4. Client can now send messages to interact with lobbies and games
5. Server sends updates as game state changes

## Message Protocol

All messages follow this JSON structure:

```json
{
  "type": "MessageType",
  "payload": {
    /* message-specific data */
  }
}
```

## Client Messages

Messages sent from client to server.

### Lobby Actions

#### CreateLobby

Create a new game lobby.

**Request:**

```json
{
  "type": "CreateLobby",
  "payload": {
    "settings": {
      "player_count": "Four",
      "turn_timeout_secs": 30,
      "allow_reconnect": true
    }
  }
}
```

**Fields:**

- `player_count`: `"Three"` or `"Four"` - Number of players for the game
- `turn_timeout_secs`: Number (default: 30) - Seconds before auto-play on timeout
- `allow_reconnect`: Boolean (default: true) - Allow players to reconnect

**Response:** `LobbyCreated`

---

#### JoinLobby

Join an existing lobby.

**Request:**

```json
{
  "type": "JoinLobby",
  "payload": {
    "lobby_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Fields:**

- `lobby_id`: UUID string - ID of the lobby to join

**Response:** `LobbyJoined` or `Error`

---

#### LeaveLobby

Leave the current lobby.

**Request:**

```json
{
  "type": "LeaveLobby"
}
```

**Response:** None (lobby is updated for other players)

---

#### StartGame

Start the game (host only).

**Request:**

```json
{
  "type": "StartGame"
}
```

**Response:** `GameStarting` or `Error`

---

#### ListLobbies

Get list of available lobbies.

**Request:**

```json
{
  "type": "ListLobbies"
}
```

**Response:** `LobbyList`

---

### Game Actions

#### PlaceBid

Place a bid during the bidding phase.

**Request:**

```json
{
  "type": "PlaceBid",
  "payload": {
    "bid": 2
  }
}
```

**Fields:**

- `bid`: Number (0 to cards dealt) - Number of tricks you predict you'll win

**Response:** `PlayerAction` broadcast to all players, or `Error`

---

#### PlayCard

Play a card during the playing phase.

**Request:**

```json
{
  "type": "PlayCard",
  "payload": {
    "card": {
      "suit": "Hearts",
      "rank": "Ace"
    }
  }
}
```

**Fields:**

- `suit`: `"Clubs"`, `"Spades"`, `"Hearts"`, or `"Diamonds"`
- `rank`: `"Two"` through `"Ten"`, `"Jack"`, `"Queen"`, `"King"`, `"Ace"`

**Response:** `PlayerAction` broadcast to all players, or `Error`

---

#### RequestGameState

Request current game state.

**Request:**

```json
{
  "type": "RequestGameState"
}
```

**Response:** `GameState`

---

### Connection

#### Ping

Keep-alive ping.

**Request:**

```json
{
  "type": "Ping"
}
```

**Response:** `Pong`

---

## Server Messages

Messages sent from server to client.

### Connection Messages

#### Connected

Sent immediately after WebSocket connection is established.

**Message:**

```json
{
  "type": "Connected",
  "payload": {
    "player_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**When Sent:** On initial connection

---

#### Pong

Response to Ping.

**Message:**

```json
{
  "type": "Pong"
}
```

**When Sent:** In response to `Ping` message

---

#### Error

Error message with description.

**Message:**

```json
{
  "type": "Error",
  "payload": {
    "message": "Lobby is full"
  }
}
```

**When Sent:** When any client action fails validation or encounters an error

---

### Lobby Messages

#### LobbyCreated

Confirmation that lobby was created.

**Message:**

```json
{
  "type": "LobbyCreated",
  "payload": {
    "lobby_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**When Sent:** After successful `CreateLobby` request

---

#### LobbyJoined

Confirmation that you joined a lobby.

**Message:**

```json
{
  "type": "LobbyJoined",
  "payload": {
    "lobby": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "host": "660e8400-e29b-41d4-a716-446655440001",
      "players": [
        "660e8400-e29b-41d4-a716-446655440001",
        "770e8400-e29b-41d4-a716-446655440002"
      ],
      "max_players": 4,
      "settings": {
        "player_count": "Four",
        "turn_timeout_secs": 30,
        "allow_reconnect": true
      }
    }
  }
}
```

**When Sent:** After successful `JoinLobby` request

---

#### LobbyUpdated

Broadcast when lobby state changes.

**Message:**

```json
{
  "type": "LobbyUpdated",
  "payload": {
    "lobby": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "host": "660e8400-e29b-41d4-a716-446655440001",
      "players": [
        "660e8400-e29b-41d4-a716-446655440001",
        "770e8400-e29b-41d4-a716-446655440002",
        "880e8400-e29b-41d4-a716-446655440003"
      ],
      "max_players": 4,
      "settings": {
        "player_count": "Four",
        "turn_timeout_secs": 30,
        "allow_reconnect": true
      }
    }
  }
}
```

**When Sent:** When a player joins or leaves the lobby

---

#### LobbyList

List of available lobbies.

**Message:**

```json
{
  "type": "LobbyList",
  "payload": {
    "lobbies": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "host": "660e8400-e29b-41d4-a716-446655440001",
        "players": ["660e8400-e29b-41d4-a716-446655440001"],
        "max_players": 4,
        "settings": {
          "player_count": "Four",
          "turn_timeout_secs": 30,
          "allow_reconnect": true
        }
      }
    ]
  }
}
```

**When Sent:** In response to `ListLobbies` request

---

#### GameStarting

Broadcast when game is starting.

**Message:**

```json
{
  "type": "GameStarting",
  "payload": {
    "game_id": "990e8400-e29b-41d4-a716-446655440000"
  }
}
```

**When Sent:** After host calls `StartGame` with sufficient players

---

### Game Messages

#### GameState

Current game state from player's perspective.

**Message:**

```json
{
  "type": "GameState",
  "payload": {
    "state": {
      "game_id": "990e8400-e29b-41d4-a716-446655440000",
      "phase": "Playing",
      "your_hand": [
        { "suit": "Hearts", "rank": "Ace" },
        { "suit": "Spades", "rank": "King" }
      ],
      "current_trick": [
        [
          "660e8400-e29b-41d4-a716-446655440001",
          { "suit": "Hearts", "rank": "Ten" }
        ]
      ],
      "scores": {
        "660e8400-e29b-41d4-a716-446655440001": 15,
        "770e8400-e29b-41d4-a716-446655440002": 10
      },
      "trump_suit": "Diamonds",
      "current_player": "770e8400-e29b-41d4-a716-446655440002",
      "your_turn": true
    }
  }
}
```

**Fields:**

- `phase`: `"Bidding"`, `"Playing"`, `"RoundComplete"`, or `"GameComplete"`
- `your_hand`: Array of cards in your hand
- `current_trick`: Array of [player_id, card] pairs for current trick
- `scores`: Map of player IDs to total scores
- `trump_suit`: Current trump suit (null during bidding)
- `current_player`: Player ID whose turn it is
- `your_turn`: Boolean indicating if it's your turn

**When Sent:**

- After game starts
- In response to `RequestGameState`
- After significant game state changes

---

#### YourTurn

Notification that it's your turn.

**Message:**

```json
{
  "type": "YourTurn",
  "payload": {
    "valid_actions": [
      { "PlayCard": { "suit": "Hearts", "rank": "Ace" } },
      { "PlayCard": { "suit": "Spades", "rank": "King" } }
    ]
  }
}
```

**When Sent:** When it becomes your turn to act

---

#### PlayerAction

Broadcast when any player takes an action.

**Message (Bid):**

```json
{
  "type": "PlayerAction",
  "payload": {
    "player_id": "660e8400-e29b-41d4-a716-446655440001",
    "action": { "Bid": 2 }
  }
}
```

**Message (PlayCard):**

```json
{
  "type": "PlayerAction",
  "payload": {
    "player_id": "660e8400-e29b-41d4-a716-446655440001",
    "action": {
      "PlayCard": { "suit": "Hearts", "rank": "Ace" }
    }
  }
}
```

**When Sent:** After any player places a bid or plays a card

---

#### TrickComplete

Broadcast when a trick is completed.

**Message:**

```json
{
  "type": "TrickComplete",
  "payload": {
    "winner": "660e8400-e29b-41d4-a716-446655440001"
  }
}
```

**When Sent:** After all players have played a card in a trick

---

#### GameOver

Broadcast when game ends.

**Message:**

```json
{
  "type": "GameOver",
  "payload": {
    "final_scores": {
      "660e8400-e29b-41d4-a716-446655440001": 125,
      "770e8400-e29b-41d4-a716-446655440002": 98,
      "880e8400-e29b-41d4-a716-446655440003": 87,
      "990e8400-e29b-41d4-a716-446655440004": 76
    }
  }
}
```

**When Sent:** When the game completes (no more cards can be dealt)

---

### Player Messages

#### PlayerJoined

Broadcast when a player joins the lobby.

**Message:**

```json
{
  "type": "PlayerJoined",
  "payload": {
    "player_id": "880e8400-e29b-41d4-a716-446655440003"
  }
}
```

**When Sent:** When a new player joins your lobby

---

#### PlayerLeft

Broadcast when a player leaves.

**Message:**

```json
{
  "type": "PlayerLeft",
  "payload": {
    "player_id": "880e8400-e29b-41d4-a716-446655440003"
  }
}
```

**When Sent:** When a player disconnects or leaves the lobby/game

---

#### PlayerReconnected

Broadcast when a disconnected player reconnects.

**Message:**

```json
{
  "type": "PlayerReconnected",
  "payload": {
    "player_id": "880e8400-e29b-41d4-a716-446655440003"
  }
}
```

**When Sent:** When a previously disconnected player reconnects to an active game

---

## Example Message Flows

### Flow 1: Creating and Starting a Game

```
Client → Server: CreateLobby
Server → Client: LobbyCreated { lobby_id: "abc123" }

Client2 → Server: JoinLobby { lobby_id: "abc123" }
Server → Client2: LobbyJoined { lobby: {...} }
Server → Client: LobbyUpdated { lobby: {...} }
Server → Client: PlayerJoined { player_id: "client2_id" }

Client → Server: StartGame
Server → All: GameStarting { game_id: "game123" }
Server → All: GameState { state: {...} }
Server → Client: YourTurn { valid_actions: [...] }
```

### Flow 2: Playing a Round

```
# Bidding Phase
Server → Player1: YourTurn { valid_actions: [Bid(0), Bid(1), ...] }
Player1 → Server: PlaceBid { bid: 2 }
Server → All: PlayerAction { player_id: "p1", action: Bid(2) }

Server → Player2: YourTurn { valid_actions: [Bid(0), Bid(1), ...] }
Player2 → Server: PlaceBid { bid: 1 }
Server → All: PlayerAction { player_id: "p2", action: Bid(1) }

# ... more bidding ...

# Playing Phase
Server → Player1: YourTurn { valid_actions: [PlayCard(...), ...] }
Player1 → Server: PlayCard { card: { suit: "Hearts", rank: "Ace" } }
Server → All: PlayerAction { player_id: "p1", action: PlayCard(...) }

# ... more plays ...

Server → All: TrickComplete { winner: "p1" }
Server → Player1: YourTurn { valid_actions: [...] }

# ... more tricks ...

Server → All: GameState { state: { phase: "RoundComplete", ... } }
Server → All: GameState { state: { phase: "Playing", ... } }  # Next round
```

### Flow 3: Player Reconnection

```
# Player disconnects
Server → Others: PlayerLeft { player_id: "p2" }

# Player reconnects within timeout
Player2 → Server: (reconnects to WebSocket)
Server → Player2: Connected { player_id: "p2" }
Server → Player2: GameState { state: {...} }  # Restore state
Server → Others: PlayerReconnected { player_id: "p2" }
```

### Flow 4: Error Handling

```
Client → Server: JoinLobby { lobby_id: "invalid" }
Server → Client: Error { message: "Lobby not found" }

Client → Server: PlayCard { card: {...} }  # Wrong suit
Server → Client: Error { message: "Must follow suit" }
```

## Data Types

### Card

```json
{
  "suit": "Hearts",
  "rank": "Ace"
}
```

**Suits:** `"Clubs"`, `"Spades"`, `"Hearts"`, `"Diamonds"`

**Ranks:** `"Two"`, `"Three"`, `"Four"`, `"Five"`, `"Six"`, `"Seven"`, `"Eight"`, `"Nine"`, `"Ten"`, `"Jack"`, `"Queen"`, `"King"`, `"Ace"`

### GamePhase

**Values:** `"Bidding"`, `"Playing"`, `"RoundComplete"`, `"GameComplete"`

### PlayerCount

**Values:** `"Three"`, `"Four"`

## Error Messages

Common error messages you may receive:

- `"Lobby not found"` - Invalid lobby ID
- `"Lobby is full"` - Cannot join, lobby at capacity
- `"Not enough players"` - Cannot start game with insufficient players
- `"Only host can start game"` - Non-host tried to start game
- `"Game not found"` - Invalid game ID
- `"Not player's turn"` - Tried to act out of turn
- `"Must follow suit"` - Played wrong suit when you have the lead suit
- `"Invalid bid"` - Bid out of range or violates last bidder rule
- `"Player not in game"` - Tried to act in a game you're not part of
