<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  export let rank: string;
  export let suit: string;
  export let playable = false;
  export let selected = false;

  const dispatch = createEventDispatcher();

  // Map API suit names to symbols if needed
  function getSuitSymbol(s: string) {
      switch(s) {
          case "Hearts": return "♥️";
          case "Diamonds": return "♦️";
          case "Clubs": return "♣️";
          case "Spades": return "♠️";
          default: return s;
      }
  }

  function getRankShort(r: string) {
      switch(r) {
            case "Two": return "2";
            case "Three": return "3";
            case "Four": return "4";
            case "Five": return "5";
            case "Six": return "6";
            case "Seven": return "7";
            case "Eight": return "8";
            case "Nine": return "9";
            case "Ten": return "10";
            case "Jack": return "J";
            case "Queen": return "Q";
            case "King": return "K";
            case "Ace": return "A";
            default: return r;
      }
  }

  $: displaySuit = getSuitSymbol(suit);
  $: displayRank = getRankShort(rank);
  $: red = displaySuit === "♥️" || displaySuit === "♦️";

  function handleClick() {
    if (playable) {
      dispatch('click');
    }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div 
  class="card" 
  class:red 
  class:playable 
  class:selected
  on:click={handleClick}
  role="button"
  tabindex="0"
>
  <div class="top-left">
    <div class="rank">{displayRank}</div>
    <div class="suit">{displaySuit}</div>
  </div>
  
  <div class="center-suit">{displaySuit}</div>
  
  <div class="bottom-right">
    <div class="rank">{displayRank}</div>
    <div class="suit">{displaySuit}</div>
  </div>
</div>

<style>
  .card {
    width: 100px;
    height: 150px;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 5px rgba(0,0,0,0.2);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 10px;
    font-family: serif;
    position: relative;
    user-select: none;
    transition: transform 0.2s, box-shadow 0.2s;
    color: black; /* Force black default */
    border: 1px solid #ccc;
  }
  
  .card.red {
    color: #e53935;
  }
  
  .top-left {
    display: flex;
    flex-direction: column;
    align-items: center;
    font-size: 1.2rem;
    line-height: 1;
  }
  
  .bottom-right {
    display: flex;
    flex-direction: column;
    align-items: center;
    font-size: 1.2rem;
    line-height: 1;
    transform: rotate(180deg);
  }
  
  .center-suit {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 2.5rem;
  }

  .card.playable:hover {
      cursor: pointer;
      transform: translateY(-20px);
      box-shadow: 0 5px 15px rgba(0,0,0,0.3);
      z-index: 10;
  }

  .card.selected {
      transform: translateY(-30px);
      box-shadow: 0 0 0 3px var(--accent, #0ea5e9);
  }
</style>
