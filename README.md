# German Bridge - Multiplayer Card Game

A real-time multiplayer German Bridge card game with WebSocket communication, user authentication, and PostgreSQL persistence.

## 🎮 What is German Bridge?

German Bridge is a trick-prediction card game for 2+ players using a standard 52-card deck. Players bid how many tricks they will win each round and score points based on making their exact bid:

- **Made bid exactly:** 10 + (tricks × tricks) points
- **Missed bid:** -((won - bid) × (won - bid)) points

The game progresses through rounds with increasing numbers of cards dealt, starting with 1 card per player and incrementing each round.

## ✨ Features

- 🎯 **Real-time multiplayer** with WebSocket communication
- 🔐 **User authentication** with JWT and Argon2 password hashing
- 💾 **PostgreSQL database** with SeaORM for data persistence
- 🎨 **Modern frontend** built with SvelteKit and TypeScript
- 🏆 **Full game rules** implementation with scoring and bidding
- 👥 **Lobby system** for matchmaking
- 🔄 **Player reconnection** support
- 📊 **Game history** tracking

## 🚀 Quick Start

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Node.js 18+** - [Install Node.js](https://nodejs.org/)
- **Docker** - [Install Docker](https://www.docker.com/) (for PostgreSQL)

### 1. Start PostgreSQL

```bash
cd backend
docker-compose up -d
```

### 2. Start Backend Server

```bash
cd backend
cargo run
```

The server will:
- Connect to PostgreSQL
- Run migrations automatically
- Start listening on `http://0.0.0.0:8080`

### 3. Start Frontend

```bash
cd frontend
npm install
npm run dev
```

The frontend will be available at `http://localhost:5173`

## 📁 Project Structure

```
german-bridge/
├── backend/           # Rust backend server
│   ├── src/
│   │   ├── entities/  # SeaORM database entities
│   │   ├── migrator/  # Database migrations
│   │   ├── handlers/  # HTTP request handlers
│   │   └── ...
│   ├── docker-compose.yml
│   └── README.md
├── frontend/          # SvelteKit frontend
│   ├── src/
│   │   ├── routes/    # SvelteKit pages
│   │   ├── lib/       # Components and stores
│   │   └── ...
│   └── README.md
└── README.md          # This file
```

## 🎮 How to Play

1. **Register/Login** - Create an account or login
2. **Create/Join Lobby** - Start a new game or join an existing one
3. **Wait for Players** - Need 3-4 players to start
4. **Bidding Phase** - Predict how many tricks you'll win
5. **Playing Phase** - Play cards and try to match your bid
6. **Scoring** - Earn points for exact bids, lose points for misses
7. **Next Round** - Continue until cards can't be evenly distributed

## 🔧 Configuration

### Backend Environment Variables

| Variable            | Description                  | Default                                                    |
| ------------------- | ---------------------------- | ---------------------------------------------------------- |
| `DATABASE_URL`      | PostgreSQL connection string | `postgres://postgres:example@localhost:5432/german_bridge` |
| `SERVER_HOST`       | Server bind address          | `0.0.0.0`                                                  |
| `SERVER_PORT`       | Server port                  | `8080`                                                     |
| `LOG_LEVEL`         | Logging level                | `info`                                                     |

## 📚 Documentation

- [Backend README](./backend/README.md) - Server setup and configuration
- [Backend API](./backend/API.md) - WebSocket and HTTP API documentation
- [Frontend README](./frontend/README.md) - Frontend development guide

## 🛠️ Development

### Backend

```bash
cd backend

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Watch mode
cargo watch -x run
```

### Frontend

```bash
cd frontend

# Development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## 🗄️ Database

The application uses PostgreSQL with SeaORM. Migrations run automatically on startup.

**Tables:**
- `users` - User accounts
- `lobbies` - Game lobbies
- `lobby_players` - Lobby membership
- `games` - Game sessions
- `game_players` - Game participation
- `game_rounds` - Round history

## 🔐 Security

- Passwords hashed with **Argon2**
- WebSocket connections protected with **JWT**
- CORS configured for frontend access
- SQL injection prevention via SeaORM

## 📝 License

[Add your license here]

## 🤝 Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## 📧 Support

For issues and questions:
- Open an issue on GitHub
- Check the documentation
- Review the API documentation
