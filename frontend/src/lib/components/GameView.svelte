<script lang="ts">
  import { onDestroy } from 'svelte';
  import { ws, type Card as CardType } from '../stores/websocket';
  import Hand from './Hand.svelte';
  import Card from './Card.svelte';
  import BidControls from './BidControls.svelte';
  import Button from './Button.svelte';
  import Scorecard from './Scorecard.svelte';

  let showHistoryModal = $state(false);
  let showRoundSummary = $state(false);
  let roundSummaryTimeout: ReturnType<typeof setTimeout> | null = null;

  const game = $derived($ws.game);
  const myPlayerId = $derived($ws.playerId);
  const isMyTurn = $derived(game?.your_turn ?? false);
  const phase = $derived(game?.phase);
  const scores = $derived(game?.scores ?? {});
  const history = $derived(game?.history ?? []);
  const players = $derived(Object.keys(scores));

  // Handle round completion with delay to show final card
  $effect(() => {
    if (game && phase === 'RoundComplete') {
      if (roundSummaryTimeout) {
        clearTimeout(roundSummaryTimeout);
      }
      roundSummaryTimeout = setTimeout(() => {
        showRoundSummary = true;
      }, 2000);
    } else {
      showRoundSummary = false;
      if (roundSummaryTimeout) {
        clearTimeout(roundSummaryTimeout);
        roundSummaryTimeout = null;
      }
    }
  });

  // Cleanup timeout on component destroy
  onDestroy(() => {
    if (roundSummaryTimeout) {
      clearTimeout(roundSummaryTimeout);
    }
  });

  // Compute valid actions
  const validBids = $derived(
    $ws.validActions
      ?.filter((a) => a.Bid !== undefined)
      .map((a) => a.Bid!.tricks) ?? []
  );

  const canPlayCard = $derived($ws.validActions?.some((a) => a.PlayCard) ?? false);

  function handleBid(bid: number) {
    ws.placeBid(bid);
  }

  function handlePlayCard(card: CardType) {
    ws.playCard(card);
  }
  
  function getPlayerName(id: string) {
    if (id === myPlayerId) return "You";
    // Get username from store's playerUsernames map
    return $ws.playerUsernames[id] || `Player ${id.slice(0, 4)}`;
  }

  function getSuitDisplay(suit: string | null | undefined) {
      if (!suit) return { icon: "Ø", color: "var(--text-secondary)" };
      switch (suit) {
          case "Hearts": return { icon: "♥️", color: "#e53935" };
          case "Diamonds": return { icon: "♦️", color: "#e53935" };
          case "Clubs": return { icon: "♣️", color: "var(--text-primary)" };
          case "Spades": return { icon: "♠️", color: "var(--text-primary)" };
          default: return { icon: suit, color: "var(--text-primary)" };
      }
  }

  function getPhaseLabel(p: string | undefined) {
      if (!p) return "";
      switch(p) {
          case "Bidding": return "Bidding";
          case "Playing": return "Playing";
          case "RoundComplete": return "Round End";
          case "GameComplete": return "Game Over";
          default: return p;
      }
  }
  const trumpDisplay = $derived(getSuitDisplay(game?.trump_suit));

  // Get current round's bid and make for the user
  const currentRound = $derived(
    game?.history ? game.history[game.history.length - 1] : undefined
  );
  const userBid = $derived($ws.currentRoundBids[myPlayerId ?? ''] ?? '-');
  const userMake = $derived($ws.currentRoundMakes[myPlayerId ?? ''] ?? '-');

  function getPlayerBidMake(pid: string) {
    const bid = $ws.currentRoundBids[pid];
    const make = $ws.currentRoundMakes[pid];
    return { 
      bid: bid !== undefined ? bid : null, 
      make: make !== undefined ? make : 0,
      hasBid: bid !== undefined 
    };
  }

  function getBidMakeStatus(bid: number | null, make: number) {
    if (bid === null) return 'pending';
    if (make < bid) return 'under';
    if (make === bid) return 'exact';
    return 'over';
  }
</script>

{#if game}
  <div class="game-container">
    <div class="header">
        <div class="scores">
            {#each Object.entries(scores) as [pid, score]}
                {#key pid}
                  {@const bidMake = getPlayerBidMake(pid)}
                  {@const status = getBidMakeStatus(bidMake.bid, bidMake.make)}
                  <div class="score-badge" class:active={game.current_player === pid}>
                      <span class="name">{getPlayerName(pid)}</span>
                      <div class="bid-make-display" class:pending={status === 'pending'} class:exact={status === 'exact'} class:over={status === 'over'} class:under={status === 'under'}>
                          <span class="make-value">{bidMake.make}</span>
                          <span class="separator">/</span>
                          <span class="bid-value">{bidMake.bid ?? '-'}</span>
                      </div>
                      <span class="score">{score}</span>
                  </div>
                {/key}
            {/each}
        </div>
        <div class="game-info">
            <div class="info-item">
                <span class="label">Phase</span>
                <div class="value-row">
                    {#if phase === 'Bidding'}
                        <!-- Speech Bubble Icon -->
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
                    {:else if phase === 'Playing'}
                        <!-- Layers/Cards Icon -->
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 2 7 12 12 22 7 12 2"></polygon><polyline points="2 17 12 22 22 17"></polyline><polyline points="2 12 12 17 22 12"></polyline></svg>
                    {:else if phase === 'RoundComplete'}
                        <!-- Flag Icon -->
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"></path><line x1="4" y1="22" x2="4" y2="15"></line></svg>
                    {:else if phase === 'GameComplete'}
                        <!-- Trophy Icon -->
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6"></path><path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18"></path><path d="M4 22h16"></path><path d="M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22"></path><path d="M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22"></path><path d="M18 2H6v7a6 6 0 0 0 12 0V2Z"></path></svg>
                    {/if}
                    <span class="value">{getPhaseLabel(phase)}</span>
                </div>
            </div>
             <div class="info-item">
                <span class="label">Trump</span>
                <div class="value-row" style="color: {trumpDisplay.color}">
                    <span class="suit-icon">{trumpDisplay.icon}</span>
                    <span class="value">{game.trump_suit ?? 'None'}</span>
                </div>
            </div>
            <Button size="sm" variant="secondary" onclick={() => (showHistoryModal = true)}>
                📊 History
            </Button>
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
        <!-- {/if} -->
        
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
    
    {#if showHistoryModal}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="overlay" onclick={() => (showHistoryModal = false)} role="button" tabindex="0">
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="history-modal" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
                <button class="history-close-btn" onclick={() => (showHistoryModal = false)}>&times;</button>
                <Scorecard {history} {players} myPlayerId={myPlayerId ?? ''} playerUsernames={$ws.playerUsernames} />
            </div>
        </div>
    {/if}
    
    {#if game && phase === 'RoundComplete' && showRoundSummary}
        <div class="overlay">
            <div class="round-summary">
                <Scorecard {history} {players} myPlayerId={myPlayerId ?? ''} playerUsernames={$ws.playerUsernames} />
                <div class="summary-footer">
                    {#if game.current_player === myPlayerId}
                        <Button variant="primary" onclick={() => ws.startNextRound()}>
                            Start Round {game.round_number + 1}
                        </Button>
                    {:else}
                        <p class="waiting-text">Waiting for {getPlayerName(game.current_player)} to start next round...</p>
                    {/if}
                </div>
            </div>
        </div>
    {/if}
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
        width: 100%;
        max-width: 100%;
        margin: 0;
        padding: 0;
        box-sizing: border-box;
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

  .bid-make-display {
      display: flex;
      align-items: center;
      gap: 4px;
      font-size: 1.5rem;
      font-weight: bold;
      font-family: monospace;
      margin: 4px 0;
      padding: 4px 8px;
      border-radius: var(--radius-sm);
      transition: all 0.3s;
  }

  .bid-make-display.pending {
      color: var(--text-primary);
      background: rgba(255, 255, 255, 0.1);
  }

  .bid-make-display.under {
      color: var(--text-primary);
      background: rgba(255, 255, 255, 0.1);
  }

  .bid-make-display.exact {
      color: #10b981;
      background: rgba(16, 185, 129, 0.15);
  }

  .bid-make-display.over {
      color: #ef4444;
      background: rgba(239, 68, 68, 0.15);
  }

  .bid-make-display .separator {
      opacity: 0.5;
      font-size: 1.2rem;
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
  
  .value-row {
      display: flex;
      align-items: center;
      gap: 6px;
  }
  
  .suit-icon {
      font-size: 1.2rem;
      line-height: 1;
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

  .overlay {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: rgba(0, 0, 0, 0.7);
      display: flex;
      justify-content: center;
      align-items: center;
      z-index: 50;
      backdrop-filter: blur(4px);
  }

  .history-modal {
      background: var(--bg-secondary);
      border-radius: var(--radius-lg);
      box-shadow: var(--shadow-xl);
      border: 1px solid var(--border-color);
      animation: slideIn 0.3s ease-out;
      max-width: 95vw;
      max-height: 90vh;
      overflow: hidden;
      display: flex;
      flex-direction: column;
      position: relative;
  }

  .history-close-btn {
      position: absolute;
      top: var(--spacing-md);
      right: var(--spacing-md);
      background: none;
      border: none;
      font-size: 1.5rem;
      cursor: pointer;
      color: var(--text-secondary);
      padding: 0;
      width: 32px;
      height: 32px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: var(--radius-sm);
      transition: all 0.2s;
      z-index: 100;
  }

  .history-close-btn:hover {
      background: var(--bg-tertiary);
      color: var(--text-primary);
  }

  .history-modal :global(> div) {
      overflow-y: auto;
      flex: 1;
  }

  .round-summary {
      background: var(--bg-secondary);
      padding: var(--spacing-xl);
      border-radius: var(--radius-lg);
      /* text-align: center;  Let Scorecard handle alignment */
      box-shadow: var(--shadow-xl);
      border: 1px solid var(--border-color);
      animation: slideIn 0.3s ease-out;
      /* Remove min-width to allow table to fit */
      max-width: 95vw;
    max-height: 90vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-lg);
  }

  .summary-footer {
      width: 100%;
      display: flex;
      justify-content: center;
      margin-top: var(--spacing-md);
  }

  .waiting-text {
      color: var(--text-secondary);
      font-style: italic;
      animation: pulse 2s infinite;
  }

  @keyframes slideIn {
      from { transform: translateY(20px); opacity: 0; }
      to { transform: translateY(0); opacity: 1; }
  }

</style>
