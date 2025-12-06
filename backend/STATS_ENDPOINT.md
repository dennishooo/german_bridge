# Server Statistics Endpoint

## Overview

The `/stats` endpoint provides real-time statistics about the server's current state, including connection and game information.

## Endpoint

```
GET /stats
```

## Response Format

The endpoint returns a JSON object with the following structure:

```json
{
  "connections": {
    "total_connections": 10,
    "active_connections": 8,
    "inactive_connections": 2
  },
  "games": {
    "active_games": 3
  }
}
```

## Fields

### Connections

- `total_connections`: Total number of player sessions (active + inactive)
- `active_connections`: Number of currently connected players
- `inactive_connections`: Number of disconnected players within reconnection timeout

### Games

- `active_games`: Number of currently active games

## Usage Example

### Using curl

```bash
curl http://localhost:8080/stats
```

### Using curl with jq for formatted output

```bash
curl -s http://localhost:8080/stats | jq .
```

### Expected Response

```json
{
  "connections": {
    "total_connections": 0,
    "active_connections": 0,
    "inactive_connections": 0
  },
  "games": {
    "active_games": 0
  }
}
```

## Implementation Details

- **ConnectionManager**: Tracks player sessions and connection states
- **GameManager**: Tracks active game instances
- Statistics are computed in real-time from in-memory data structures
- All operations are thread-safe using async RwLocks

## Monitoring Use Cases

1. **Health Monitoring**: Check if the server is handling connections and games
2. **Load Monitoring**: Track the number of active games and connections
3. **Capacity Planning**: Understand current server utilization
4. **Debugging**: Verify that connections and games are being properly cleaned up
