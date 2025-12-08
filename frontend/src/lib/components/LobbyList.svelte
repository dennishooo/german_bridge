<script lang="ts">
  import { ws, type Lobby } from '../stores/websocket';
  import Button from './Button.svelte';
  import Input from './Input.svelte';

  let newLobbySettings = {
    player_count: 4,
    turn_timeout_secs: 30,
    allow_reconnect: true
  };

  let joinLobbyId = "";

  function createLobby() {
    ws.createLobby(newLobbySettings);
  }

  function joinLobby(id: string) {
    if (!id) return;
    ws.joinLobby(id);
  }
</script>

<div class="lobby-list-container">
  <div class="section">
    <h2>Available Lobbies</h2>
    <div class="lobbies">
      {#if $ws.lobbies.length === 0}
        <p class="empty-msg">No lobbies found. Create one!</p>
      {:else}
        {#each $ws.lobbies as lobby}
          <div class="lobby-card">
            <div class="lobby-info">
              <h3>Lobby {lobby.id.slice(0, 8)}...</h3>
              <p>Host: {lobby.host.slice(0, 8)}...</p>
              <p>Players: {lobby.players.length}</p>
            </div>
      <Button onclick={() => joinLobby(lobby.id)} disabled={lobby.players.length >= lobby.max_players}>
              {lobby.players.length >= lobby.max_players ? 'Full' : 'Join'}
            </Button>
          </div>
        {/each}
      {/if}
    </div>
    <div class="refresh-btn">
        <Button variant="secondary" onclick={() => ws.listLobbies()}>Refresh List</Button>
    </div>
  </div>

  <div class="divider"></div>

  <div class="section form-section">
    <h2>Create Lobby</h2>


    <div class="form-group">
        <label for="timeout">Turn Timeout (sec)</label>
        <input type="number" id="timeout" bind:value={newLobbySettings.turn_timeout_secs} min="10" max="120" />
    </div>

    <div class="actions">
        <Button onclick={createLobby}>Create Lobby</Button>
    </div>
  </div>
  
  <div class="divider"></div>

   <div class="section form-section">
    <h2>Join by ID</h2>
    <div class="actions">
        <Input placeholder="Lobby ID" bind:value={joinLobbyId} />
        <Button onclick={() => joinLobby(joinLobbyId)} disabled={!joinLobbyId}>Join</Button>
    </div>
  </div>
</div>

<style>
  .lobby-list-container {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xl);
    max-width: 600px;
    margin: 0 auto;
    width: 100%;
  }

  .section {
    background: var(--bg-secondary);
    padding: var(--spacing-lg);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-md);
  }

  h2 {
    margin-bottom: var(--spacing-md);
    font-size: 1.5rem;
    color: var(--text-primary);
  }

  .lobbies {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }

  .lobby-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
  }

  .lobby-info h3 {
    margin: 0 0 var(--spacing-xs);
    font-size: 1.1rem;
  }
  
  .lobby-info p {
    margin: 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .empty-msg {
    text-align: center;
    color: var(--text-secondary);
    font-style: italic;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-md);
  }

  label {
    font-weight: 600;
    font-size: 0.9rem;
  }

  input {
    padding: var(--spacing-sm);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .actions {
    display: flex;
    gap: var(--spacing-md);
  }
  
  .divider {
      height: 1px;
      background: var(--border-color);
      width: 100%;
  }
  
  .refresh-btn {
      margin-top: var(--spacing-md);
      display: flex;
      justify-content: center;
  }
</style>
