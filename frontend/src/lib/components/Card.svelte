<script lang="ts">
  export let rank: string;
  export let suit: string;
  
  $: isRed = suit === '♥️' || suit === '♦️';
</script>

<div class="card">
  <div class="rank" class:red={isRed}>{rank}</div>
  <div class="suit" class:red={isRed}>{suit}</div>
</div>

<style>
  .card {
    width: 100px;
    height: 140px;
    background-color: var(--bg-secondary);
    border: 2px solid var(--border-color);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    box-shadow: var(--shadow-md);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
    cursor: pointer;
  }
  
  .card:hover {
    transform: translateY(-4px);
    box-shadow: var(--shadow-lg);
  }
  
  .rank {
    font-size: 1.5rem;
    font-weight: bold;
    color: var(--text-primary);
  }
  
  .suit {
    font-size: 2rem;
  }
  
  .red {
    color: #ef4444;
  }
</style>
