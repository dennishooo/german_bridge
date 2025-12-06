<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { ws } from '../lib/stores/websocket';
  import LobbyList from '../lib/components/LobbyList.svelte';
  import LobbyView from '../lib/components/LobbyView.svelte';
  import GameView from '../lib/components/GameView.svelte';
  import ThemeToggle from '../lib/components/ThemeToggle.svelte';
  import Button from '../lib/components/Button.svelte';

  // Auto-connect on mount
  onMount(() => {
    ws.connect();
  });

  $: connected = $ws.connected;
  $: lobby = $ws.lobby;
  $: game = $ws.game;
  $: error = $ws.error;
</script>

<div class="page">
  <header class="header">
    <h1 class="title">German Bridge</h1>
    <div class="header-controls">
        {#if connected}
            <span class="status connected">Connected</span>
            {#if $ws.playerId}
                <span class="player-id" title={$ws.playerId}>ID: {$ws.playerId.slice(0, 4)}...</span>
            {/if}
        {:else}
            <span class="status disconnected">Disconnected</span>
            <Button size="sm" on:click={() => ws.connect()}>Connect</Button>
        {/if}
        <ThemeToggle />
    </div>
  </header>
  
  <main class="container">
    {#if error}
        <div class="error-banner">
            {error}
            <button class="close-btn" on:click={() => $ws.error = null}>&times;</button>
        </div>
    {/if}

    {#if !connected}
       <div class="welcome">
           <h2>Welcome to German Bridge</h2>
           <p>Connecting to server...</p>
       </div>
    {:else if game}
        <GameView />
    {:else if lobby}
        <LobbyView />
    {:else}
        <LobbyList />
    {/if}
  </main>
</div>

<style>
  .page {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md) var(--spacing-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }
  
  .header-controls {
      display: flex;
      align-items: center;
      gap: var(--spacing-md);
  }
  
  .status {
      font-size: 0.8rem;
      font-weight: 600;
      padding: 2px 8px;
      border-radius: var(--radius-full);
  }
  
  .status.connected {
      background: var(--color-success);
      color: white;
  }
  
  .status.disconnected {
      background: var(--color-error);
      color: white;
  }

  .player-id {
      font-family: monospace;
      font-size: 0.8rem;
      background: var(--bg-tertiary);
      padding: 2px 6px;
      border-radius: var(--radius-sm);
      color: var(--text-secondary);
  }
  
  .title {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }
  
  .container {
    flex: 1;
    padding: var(--spacing-lg);
    overflow: hidden; /* For game view */
  }
  
  .error-banner {
      background: var(--color-error);
      color: white;
      padding: var(--spacing-md);
      border-radius: var(--radius-md);
      margin-bottom: var(--spacing-md);
      display: flex;
      justify-content: space-between;
      align-items: center;
  }
  
  .close-btn {
      background: none;
      border: none;
      color: white;
      font-size: 1.5rem;
      cursor: pointer;
  }
  
  .welcome {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      color: var(--text-secondary);
  }
</style>
