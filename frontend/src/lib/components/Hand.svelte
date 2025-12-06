<script lang="ts">
  import type { Card as CardType } from '../stores/websocket';
  import Card from './Card.svelte';

  export let hand: CardType[] = [];
  export let validActions: any[] | null = null;
  export let onPlayCard: (card: CardType) => void;

  function isCardValid(card: CardType): boolean {
    if (!validActions) return false;
    return validActions.some(action => 
      action.PlayCard && 
      action.PlayCard.suit === card.suit && 
      action.PlayCard.rank === card.rank
    );
  }

  function handleCardClick(card: CardType) {
    if (isCardValid(card)) {
      onPlayCard(card);
    }
  }
</script>

<div class="hand">
  <div class="cards">
    {#each hand as card}
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div 
        class="card-wrapper" 
        class:valid={isCardValid(card)}
        on:click={() => handleCardClick(card)}
      >
        <Card rank={card.rank} suit={card.suit} />
        {#if !isCardValid(card) && validActions}
            <div class="overlay"></div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .hand {
    display: flex;
    justify-content: center;
    padding: var(--spacing-md);
    overflow-x: auto;
  }

  .cards {
    display: flex;
    gap: var(--spacing-xs); /* Use negative gap for overlap effect? Or small gap. */
    align-items: center;
  }
  
  /* Overlap effect */
  .cards {
      gap: 0;
  }
  
  .card-wrapper {
      transition: transform 0.2s ease;
      cursor: pointer;
      position: relative;
      margin-left: -30px; /* Overlap */
  }
  
  .card-wrapper:first-child {
      margin-left: 0;
  }
  
  .card-wrapper:hover {
      transform: translateY(-20px);
      z-index: 10;
  }
  
  .card-wrapper.valid:hover {
      transform: translateY(-30px) scale(1.05);
  }

  .overlay {
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0,0,0,0.3);
      border-radius: var(--radius-md);
      pointer-events: none;
  }
</style>
