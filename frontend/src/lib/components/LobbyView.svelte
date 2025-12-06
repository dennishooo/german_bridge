<script lang="ts">
  import { ws } from '../stores/websocket';
  import Button from './Button.svelte';

  $: lobby = $ws.lobby;
  $: isHost = lobby && lobby.host === $ws.playerId;
  $: canStart = isHost && lobby && lobby.players.length === lobby.max_players; // Or maybe min 3?
  // API says 3 or 4 players. Settings dictate max.
  // Actually settings.player_count is "Three" or "Four".
  // So we need exact match usually?
  // Let's assume match.

  function startGame() {
    ws.startGame();
  }

  function leaveLobby() {
    ws.leaveLobby();
  }
</script>

<div class="lobby-view">
  {#if lobby}
    <div class="header">
      <h2>Lobby</h2>
      <span class="lobby-id">ID: {lobby.id}</span>
    </div>

    <div class="info-card">
      <div class="setting">
        <span class="label">Players:</span>
        <span class="value">{lobby.settings.player_count}</span>
      </div>
      <div class="setting">
        <span class="label">Timeout:</span>
        <span class="value">{lobby.settings.turn_timeout_secs}s</span>
      </div>
      <div class="setting">
        <span class="label">Status:</span>
        <span class="value">{lobby.players.length} / {lobby.max_players} Joined</span>
      </div>
    </div>

    <div class="players-list">
      <h3>Players</h3>
      <ul>
        {#each lobby.players as player}
          <li class:me={player === $ws.playerId} class:host={player === lobby.host}>
            <span class="name">Player {player.slice(0, 8)}...</span>
            <div class="badges">
                {#if player === $ws.playerId}<span class="badge me">You</span>{/if}
                {#if player === lobby.host}<span class="badge host">Host</span>{/if}
            </div>
          </li>
        {/each}
      </ul>
    </div>

    <div class="actions">
      {#if isHost}
        <Button on:click={startGame} disabled={!canStart} variant="primary">
          Start Game
        </Button>
      {:else}
        <div class="waiting-msg">Waiting for host to start...</div>
      {/if}
      <Button on:click={leaveLobby} variant="secondary">Leave Lobby</Button>
    </div>
  {:else}
    <p>Loading lobby...</p>
  {/if}
</div>

<style>
  .lobby-view {
    max-width: 600px;
    margin: 0 auto;
    padding: var(--spacing-lg);
    background: var(--bg-secondary);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-md);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
    border-bottom: 2px solid var(--border-color);
    padding-bottom: var(--spacing-md);
  }

  .lobby-id {
    font-family: monospace;
    background: var(--bg-tertiary);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    font-size: 0.9rem;
  }

  .info-card {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--spacing-md);
    background: var(--bg-tertiary);
    padding: var(--spacing-md);
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-lg);
  }

  .setting {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .label {
    font-size: 0.8rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .value {
    font-weight: 700;
    font-size: 1.1rem;
  }

  .players-list ul {
    list-style: none;
    padding: 0;
    margin: 0 0 var(--spacing-lg) 0;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .players-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    background: var(--bg-primary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
  }

  .badges {
    display: flex;
    gap: var(--spacing-xs);
  }

  .badge {
    padding: 2px 8px;
    border-radius: var(--radius-full);
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .badge.me {
    background: var(--color-primary-100);
    color: var(--color-primary-800);
  }

  .badge.host {
    background: var(--color-warning);
    color: white; /* Warning is usually light, but let's check vars. #f59e0b */
    /* Text on warning might need to be black or white depending. White is safer on deep orange. */
  }

  .actions {
    display: flex;
    justify-content: center;
    gap: var(--spacing-md);
    align-items: center;
  }

  .waiting-msg {
    color: var(--text-secondary);
    font-style: italic;
  }
</style>
