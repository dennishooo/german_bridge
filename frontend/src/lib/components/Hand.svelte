<script lang="ts">
  import type { Card as CardType } from '../stores/websocket';
  import Card from './Card.svelte';

  const { hand = [], validActions = null, onPlayCard } = $props<{ hand?: CardType[]; validActions?: Array<any> | null; onPlayCard: (card: CardType) => void }>();

  // Suit order (from left to right)
  const suitOrder = { Clubs: 0, Diamonds: 1, Hearts: 2, Spades: 3 };
  
  // Rank order (from lowest to highest)
  const rankOrder = {
    Two: 0,
    Three: 1,
    Four: 2,
    Five: 3,
    Six: 4,
    Seven: 5,
    Eight: 6,
    Nine: 7,
    Ten: 8,
    Jack: 9,
    Queen: 10,
    King: 11,
    Ace: 12,
  };

  function sortCards(cards: CardType[]): CardType[] {
    return [...cards].sort((a, b) => {
      const suitDiff = suitOrder[a.suit as keyof typeof suitOrder] - suitOrder[b.suit as keyof typeof suitOrder];
      if (suitDiff !== 0) return suitDiff;
      return rankOrder[a.rank as keyof typeof rankOrder] - rankOrder[b.rank as keyof typeof rankOrder];
    });
  }

  function isCardValid(card: CardType): boolean {
    if (!validActions) return false;
    return (validActions as Array<any>).some((action: any) => 
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

  // Svelte 5 idiomatic: $derived for computed values
  const sortedHand = $derived(sortCards(hand));
</script>

<div class="hand">
  <div class="cards">
    {#each sortedHand as card}
      <div 
        class="card-wrapper" 
        class:valid={isCardValid(card)}
        tabindex="0"
        role="button"
        aria-pressed="false"
        onclick={() => handleCardClick(card)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleCardClick(card); }}
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
      z-index: 100;
  }
  
  .card-wrapper.valid:hover {
      transform: translateY(-30px) scale(1.05);
      z-index: 100;
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
