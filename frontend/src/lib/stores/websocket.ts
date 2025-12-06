import { writable, get } from 'svelte/store';

// --- Types based on API.md ---

export type Suit = "Clubs" | "Spades" | "Hearts" | "Diamonds";
export type Rank = "Two" | "Three" | "Four" | "Five" | "Six" | "Seven" | "Eight" | "Nine" | "Ten" | "Jack" | "Queen" | "King" | "Ace";

export interface Card {
    suit: Suit;
    rank: Rank;
}

export type GamePhase = "Bidding" | "Playing" | "RoundComplete" | "GameComplete";

export interface LobbySettings {
    player_count: "Three" | "Four";
    turn_timeout_secs: number;
    allow_reconnect: boolean;
}

export interface Lobby {
    id: string;
    host: string;
    players: string[];
    max_players: number;
    settings: LobbySettings;
}

export interface GameState {
    game_id: string;
    phase: GamePhase;
    your_hand: Card[];
    current_trick: [string, Card][];
    scores: Record<string, number>;
    trump_suit: Suit | null;
    current_player: string;
    your_turn: boolean;
}

export interface ValidAction {
    PlayCard?: Card;
    Bid?: { tricks: number };
}

// --- Store State ---

export interface AppState {
    connected: boolean;
    playerId: string | null;
    lobby: Lobby | null;
    game: GameState | null;
    lobbies: Lobby[]; // For the lobby list
    validActions: ValidAction[] | null; // actions valid for *your* turn
    error: string | null;
}

const initialState: AppState = {
    connected: false,
    playerId: null,
    lobby: null,
    game: null,
    lobbies: [],
    validActions: null,
    error: null,
};

export const ws = createWebSocketStore();

function createWebSocketStore() {
    const { subscribe, update, set } = writable<AppState>(initialState);
    let ws: WebSocket | null = null;
    let pingInterval: ReturnType<typeof setInterval>;

    function connect() {
        if (ws) return;

        const url = 'ws://localhost:8080/ws';
        ws = new WebSocket(url);

        ws.onopen = () => {
            console.log('Connected to WebSocket');
            // Start pinging to keep connection alive
            pingInterval = setInterval(() => {
                send('Ping');
            }, 30000);
        };

        ws.onmessage = (event) => {
            try {
               const msg = JSON.parse(event.data);
               handleMessage(msg);
            } catch (e) {
                console.error("Failed to parse message:", event.data);
            }
        };

        ws.onclose = () => {
            console.log('Disconnected');
            ws = null;
            clearInterval(pingInterval);
            update(s => ({ ...initialState, error: "Disconnected from server" }));
        };

        ws.onerror = (err) => {
            console.error('WebSocket error:', err);
            update(s => ({ ...s, error: "Connection error" }));
        };
    }

    function send(type: string, payload?: any) {
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type, payload }));
        } else {
            console.warn("Cannot send message, not connected");
        }
    }

    function handleMessage(msg: any) {
        console.log("Received:", msg);
        update(state => {
            const newState = { ...state, error: null };

            switch (msg.type) {
                case 'Connected':
                    newState.connected = true;
                    newState.playerId = msg.payload.player_id;
                    send('ListLobbies');
                    break;
                case 'Pong':
                    break;
                case 'Error':
                    newState.error = msg.payload.message;
                    break;
                
                // Lobby Messages
                case 'LobbyCreated':
                    // Refresh lobby list so others can see it immediately (and us if join fails)
                    send('ListLobbies');
                    // Auto-join the lobby we just created
                    if (msg.payload.lobby_id) {
                        send('JoinLobby', { lobby_id: msg.payload.lobby_id });
                    }
                    break;
                case 'LobbyJoined':
                    newState.lobby = msg.payload.lobby;
                    newState.validActions = null; 
                    newState.game = null; 
                    break;
                case 'LobbyUpdated':
                    if (newState.lobby && newState.lobby.id === msg.payload.lobby.id) {
                        newState.lobby = msg.payload.lobby;
                    }
                    break;
                case 'LobbyList':
                    newState.lobbies = msg.payload.lobbies;
                    break;
                
                // Game Messages
                case 'GameStarting':
                    // We need to request the game state to transition to the game view
                    send('RequestGameState');
                    break;
                case 'GameState':
                    newState.game = msg.payload.state;
                    break;
                case 'YourTurn':
                    newState.validActions = msg.payload.valid_actions;
                    if (newState.game) newState.game.your_turn = true;
                    break;
                case 'PlayerAction':
                    const { player_id, action, next_player } = msg.payload;
                    if (newState.game) {
                        // Update current_player
                        if (next_player) newState.game.current_player = next_player;

                        // Handle PlayCard
                        if (action.PlayCard) {
                            const card = action.PlayCard;
                            
                            // Check if the current trick is full (based on lobby settings or default 4)
                            // If full, we assume this new card starts a new trick, so we clear the old one.
                            // We use >= max_players to be safe.
                            const maxPlayers = newState.lobby?.max_players || 4;
                            if (newState.game.current_trick.length >= maxPlayers) {
                                newState.game.current_trick = [];
                            }

                            // Add to current trick
                            newState.game.current_trick = [...newState.game.current_trick, [player_id, card]];
                            
                            // If it's me, remove from hand
                            if (player_id === newState.playerId) {
                                newState.game.your_hand = newState.game.your_hand.filter(
                                    c => c.suit !== card.suit || c.rank !== card.rank
                                );
                                newState.validActions = null;
                                newState.game.your_turn = false;
                            }
                        }
                    }
                    break;
                case 'TrickComplete':
                    // We don't clear the trick here anymore.
                    // We leave it visible until the next card is played (see PlayerAction above)
                    // or until the round ends (GameState update).
                    break;
                case 'GameOver':
                    // Final scores are in payload
                    break;
                
                // Player Events
                case 'PlayerJoined':
                case 'PlayerLeft':
                case 'PlayerReconnected':
                     // These trigger LobbyUpdated
                     break;
            }
            return newState;
        });
    }

    return {
        subscribe,
        connect,
        createLobby: (settings: LobbySettings) => send('CreateLobby', { settings }),
        joinLobby: (lobby_id: string) => send('JoinLobby', { lobby_id }),
        leaveLobby: () => {
            send('LeaveLobby');
            update(s => ({ ...s, lobby: null, game: null }));
            // Refresh lobby list after leaving
            setTimeout(() => send('ListLobbies'), 100);
        },
        startGame: () => send('StartGame'),
        listLobbies: () => send('ListLobbies'),
        placeBid: (bid: number) => {
            send('PlaceBid', { bid: { tricks: bid } });
            // Optimistically hide the bid controls
            update(s => {
                if (s.game) {
                    return { ...s, validActions: null, game: { ...s.game, your_turn: false } };
                }
                return s;
            });
        },
        playCard: (card: Card) => send('PlayCard', { card }),
        requestGameState: () => send('RequestGameState'),
        ping: () => send('Ping')
    };
}
