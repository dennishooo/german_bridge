<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Card from "../lib/components/Card.svelte";
  import ThemeToggle from "../lib/components/ThemeToggle.svelte";

  let ranks = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];
  let suits = ["♠️", "♥️", "♦️", "♣️"];
  let cards = [];
  for (let suit of suits) {
    for (let rank of ranks) {
      cards.push({ rank, suit });
    }
  }
</script>

<div class="page">
  <header class="header">
    <h1 class="title">German Bridge</h1>
    <ThemeToggle />
  </header>
  
  <main class="container">
    {#each cards as card}
      <Card rank={card.rank} suit={card.suit} />
    {/each}
  </main>
</div>

<style>
  .page {
    min-height: 100vh;
    padding: var(--spacing-lg);
  }
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xl);
  }
  
  .title {
    font-size: 2rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }
  
  .container {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: var(--spacing-md);
    max-width: 1400px;
    margin: 0 auto;
  }
</style>
