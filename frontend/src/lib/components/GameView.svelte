<script lang="ts">
  import { ws, type Card as CardType } from '../stores/websocket';
  import Hand from './Hand.svelte';
  import Card from './Card.svelte';
  import BidControls from './BidControls.svelte';
  import Button from './Button.svelte';

  $: game = $ws.game;
  $: myPlayerId = $ws.playerId;
  $: isMyTurn = game?.your_turn;
  $: phase = game?.phase;
  $: scores = game?.scores ?? {};
  
  // Compute valid actions
  $: validBids = $ws.validActions
      ?.filter(a => a.Bid !== undefined)
      .map(a => a.Bid!.tricks) ?? [];
      
  $: canPlayCard = $ws.validActions?.some(a => a.PlayCard);

  function handleBid(bid: number) {
    ws.placeBid(bid);
  }

  function handlePlayCard(card: CardType) {
    ws.playCard(card);
  }
  
  function getPlayerName(id: string) {
      if (id === myPlayerId) return "You";
      // In a real app we might have a map of names
      return `Player ${id.slice(0, 4)}`;
  }
</script>

{#if game}
  <div class="game-container">
    <div class="header">
        <div class="scores">
            {#each Object.entries(scores) as [pid, score]}
                <div class="score-badge" class:active={game.current_player === pid}>
                    <span class="name">{getPlayerName(pid)}</span>
                    <span class="score">{score}</span>
                </div>
            {/each}
        </div>
        <div class="game-info">
            <div class="info-item">
                <span class="label">Phase</span>
                <span class="value">{phase}</span>
            </div>
             <div class="info-item">
                <span class="label">Trump</span>
                <span class="value">{game.trump_suit ?? 'None'}</span>
            </div>
        </div>
    </div>

    <div class="board">
        <!-- Trick Area -->
        <div class="trick-area">
             {#if game.current_trick.length === 0}
                <div class="empty-trick">Waiting for play...</div>
             {:else}
                {#each game.current_trick as [pid, card], i}
                    <div class="played-card" style="--rotation: {i * 20 - (game.current_trick.length-1)*10}deg">
                        <span class="player-label">{getPlayerName(pid)}</span>
                        <Card rank={card.rank} suit={card.suit} />
                    </div>
                {/each}
             {/if}
        </div>
        
        <div class="status-message">
            {#if isMyTurn}
                <div class="turn-indicator">It's your turn!</div>
            {:else}
                <div class="waiting-indicator">Waiting for {getPlayerName(game.current_player)}...</div>
            {/if}
        </div>
    </div>

    <div class="controls-area">
        {#if phase === 'Bidding' && isMyTurn}
            <div class="bidding-modal">
                <h3>Place your bid</h3>
                <BidControls validBids={validBids} onBid={handleBid} />
            </div>
        {/if}
        
        <Hand 
            hand={game.your_hand} 
            validActions={$ws.validActions} 
            onPlayCard={handlePlayCard} 
        />
    </div>
  </div>
{:else}
  <div class="loading">Loading game state...</div>
{/if}

<style>
  .game-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    max-height: 100vh;
    background: var(--bg-primary);
    overflow: hidden;
  }
  
  .header {
      padding: var(--spacing-md);
      background: var(--bg-secondary);
      border-bottom: 1px solid var(--border-color);
      display: flex;
      justify-content: space-between;
      align-items: center;
  }
  
  .scores {
      display: flex;
      gap: var(--spacing-md);
  }
  
  .score-badge {
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: var(--spacing-xs) var(--spacing-sm);
      background: var(--bg-tertiary);
      border-radius: var(--radius-md);
      border: 2px solid transparent;
  }
  
  .score-badge.active {
      border-color: var(--color-warning);
      box-shadow: 0 0 10px rgba(245, 158, 11, 0.3);
  }
  
  .score-badge .name {
      font-size: 0.75rem;
      color: var(--text-secondary);
  }
  
  .score-badge .score {
      font-weight: bold;
      font-size: 1.1rem;
  }
  
  .game-info {
      display: flex;
      gap: var(--spacing-lg);
  }
  
  .info-item {
      display: flex;
      flex-direction: column;
      align-items: flex-end;
  }
  
  .info-item .label {
      font-size: 0.75rem;
      color: var(--text-secondary);
      text-transform: uppercase;
  }
  
  .info-item .value {
      font-weight: bold;
  }
  
  .board {
      flex: 1;
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: center;
      position: relative;
      background: radial-gradient(circle at center, var(--bg-secondary) 0%, var(--bg-primary) 100%);
  }
  
  .trick-area {
      position: relative;
      width: 300px;
      height: 200px;
      display: flex;
      justify-content: center;
      align-items: center;
  }
  
  .empty-trick {
      color: var(--text-tertiary);
      font-style: italic;
      border: 2px dashed var(--border-color);
      padding: var(--spacing-lg);
      border-radius: var(--radius-lg);
  }
  
  .played-card {
      position: absolute;
      transform: rotate(var(--rotation));
      transition: all 0.5s ease-out;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: var(--spacing-xs);
  }
  
  .player-label {
      font-size: 0.75rem;
      background: rgba(0,0,0,0.5);
      color: white;
      padding: 2px 6px;
      border-radius: var(--radius-sm);
  }
  
  .status-message {
      margin-top: var(--spacing-xl);
      height: 40px;
  }
  
  .turn-indicator {
      background: var(--color-success);
      color: white;
      padding: var(--spacing-sm) var(--spacing-lg);
      border-radius: var(--radius-full);
      font-weight: bold;
      animation: pulse 2s infinite;
  }
  
  @keyframes pulse {
      0% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.7); }
      70% { box-shadow: 0 0 0 10px rgba(16, 185, 129, 0); }
      100% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0); }
  }
  
  .waiting-indicator {
     color: var(--text-secondary); 
  }

  .controls-area {
      padding: var(--spacing-md);
      background: var(--bg-secondary);
      border-top: 1px solid var(--border-color);
      position: relative;
      min-height: 200px;
      display: flex;
      flex-direction: column;
      justify-content: flex-end;
  }
  
  .bidding-modal {
      position: absolute;
      bottom: 100%;
      left: 50%;
      transform: translateX(-50%);
      background: var(--bg-primary);
      padding: var(--spacing-lg);
      border-radius: var(--radius-lg) var(--radius-lg) 0 0;
      box-shadow: var(--shadow-lg);
      border: 1px solid var(--border-color);
      border-bottom: none;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: var(--spacing-md);
      z-index: 20;
      width: 100%;
      max-width: 500px;
  }
  
  .bidding-modal h3 {
      margin: 0;
  }
  
  .loading {
      display: flex;
      justify-content: center;
      align-items: center;
      height: 100vh;
      font-size: 1.5rem;
      color: var(--text-secondary);
  }
</style>
