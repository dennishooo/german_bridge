<script lang="ts">
  import Button from './Button.svelte';
  const { validBids = [], onBid } = $props<{ validBids?: number[]; onBid: (bid: number) => void }>();
</script>

<div class="bid-controls">
  <h3>Place Your Bid</h3>
  <div class="bids-grid">
    {#each validBids as bid}
      <Button variant="secondary" onclick={() => onBid(bid)}>
        {bid}
      </Button>
    {/each}
  </div>
</div>

<style>
  .bid-controls {
    background: var(--bg-tertiary);
    padding: var(--spacing-md);
    border-radius: var(--radius-md);
    text-align: center;
    position: fixed;
    bottom: 20%;
    left: 50%;
    transform: translateX(-50%);
    box-shadow: var(--shadow-xl);
    z-index: 100;
  }

  h3 {
    margin-bottom: var(--spacing-md);
  }

  .bids-grid {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
    justify-content: center;
    max-width: 400px;
  }
</style>
