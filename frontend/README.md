# German Bridge Frontend

Modern SvelteKit frontend for the German Bridge multiplayer card game.

## Features

- 🎨 **Modern UI** with responsive design
- 🔐 **Authentication** - Login and registration
- 🎮 **Real-time gameplay** via WebSocket
- 👥 **Lobby system** for matchmaking
- 📊 **Live scorecard** with round history
- 🎯 **Interactive bidding** and card playing
- 👤 **Username display** instead of session IDs

## Tech Stack

- **SvelteKit** - Frontend framework
- **TypeScript** - Type safety
- **WebSocket** - Real-time communication
- **Vite** - Build tool

## Getting Started

### Prerequisites

- Node.js 18 or higher
- Running backend server (see [backend README](../backend/README.md))

### Installation

```bash
npm install
```

### Development

```bash
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

### Build

```bash
npm run build
```

### Preview Production Build

```bash
npm run preview
```

## Project Structure

```
frontend/
├── src/
│   ├── routes/              # SvelteKit pages
│   │   ├── +page.svelte    # Main game page
│   │   └── auth/           # Authentication pages
│   ├── lib/
│   │   ├── components/     # Svelte components
│   │   │   ├── Auth.svelte
│   │   │   ├── LobbyView.svelte
│   │   │   ├── GameView.svelte
│   │   │   └── ...
│   │   └── stores/         # State management
│   │       └── websocket.ts
│   └── app.css             # Global styles
├── static/                 # Static assets
└── package.json
```

## WebSocket Store

The `websocket.ts` store manages all real-time communication:

```typescript
import { ws } from '$lib/stores/websocket';

// Subscribe to state
$ws.connected
$ws.playerId
$ws.username
$ws.lobby
$ws.game

// Send messages
ws.createLobby(settings);
ws.joinLobby(lobbyId);
ws.placeBid(bid);
ws.playCard(card);
```

## Components

### Auth.svelte
Login and registration forms with JWT token management.

### LobbyView.svelte
Lobby interface showing players, settings, and start game button.

### GameView.svelte
Main game interface with:
- Player hand
- Current trick
- Bidding controls
- Scorecard
- Round history

### Scorecard.svelte
Displays round-by-round scoring with bids, tricks won, and points.

## Environment Variables

Create a `.env` file:

```
PUBLIC_WS_URL=ws://localhost:8080/ws
PUBLIC_API_URL=http://localhost:8080/api
```

## Development Tips

### Hot Module Replacement
Changes to `.svelte` files reload instantly during development.

### TypeScript
The project uses TypeScript for type safety. Check types with:
```bash
npm run check
```

### Debugging
Use browser DevTools to inspect WebSocket messages:
1. Open DevTools → Network tab
2. Filter by WS
3. Click on the WebSocket connection
4. View Messages tab

## Troubleshooting

### WebSocket Connection Failed
- Ensure backend server is running on port 8080
- Check that you're logged in (JWT token in localStorage)
- Verify CORS settings in backend

### Login Issues
- Check backend is running
- Verify DATABASE_URL is set correctly
- Check browser console for errors

## Contributing

See main [README](../README.md) for contribution guidelines.
