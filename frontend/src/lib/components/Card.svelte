<script lang="ts">
  const { rank, suit, playable = false, selected = false, onclick } = $props<{
    rank: string;
    suit: string;
    playable?: boolean;
    selected?: boolean;
    onclick?: () => void;
  }>();

  // Map API suit names to symbols
  function getSuitSymbol(s: string) {
      switch(s) {
          case "Hearts": return "♥";
          case "Diamonds": return "♦";
          case "Clubs": return "♣";
          case "Spades": return "♠";
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

  const displaySuit = $derived(getSuitSymbol(suit));
  const displayRank = $derived(getRankShort(rank));
  const isRed = $derived(suit === "Hearts" || suit === "Diamonds");

  function handleClick() {
    if (playable && onclick) {
      onclick();
    }
  }
</script>

<div 
  class="card" 
  class:red={isRed}
  class:playable 
  class:selected
  onclick={handleClick}
  role="button"
  tabindex="0"
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleClick(); }}
>
  <div class="corner corner-top">
    <div class="rank">{displayRank}</div>
    <div class="suit">{displaySuit}</div>
  </div>
  
  <div class="center-suit">{displaySuit}</div>
  
  <div class="corner corner-bottom">
    <div class="rank">{displayRank}</div>
    <div class="suit">{displaySuit}</div>
  </div>
</div>

<style>
  .card {
    width: 100px;
    height: 140px;
    aspect-ratio: 5 / 7;
    background: #ffffff;
    border-radius: var(--radius-lg);
    box-shadow: 
      0 1px 3px rgba(0, 0, 0, 0.08),
      0 2px 6px rgba(0, 0, 0, 0.04);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 10px 8px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', sans-serif;
    position: relative;
    user-select: none;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    color: #1a1a1a;
    border: 1px solid var(--border-color);
    cursor: default;
  }
  
  .card.red {
    color: var(--color-error);
  }
  
  .corner {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    font-weight: 600;
    line-height: 1.1;
    z-index: 1;
  }

  .corner-top {
    align-self: flex-start;
  }

  .corner-bottom {
    align-self: flex-end;
    transform: rotate(180deg);
  }
  
  .rank {
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  
  .suit {
    font-size: 18px;
    line-height: 1;
    margin-top: -2px;
  }
  
  .center-suit {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 48px;
    font-weight: 300;
    color: currentColor;
  }

  .card.playable {
    cursor: pointer;
  }

  .card.playable:hover {
    transform: translateY(-8px) scale(1.05);
    box-shadow: 
      0 4px 12px rgba(0, 0, 0, 0.12),
      0 8px 24px rgba(0, 0, 0, 0.08);
    z-index: 10;
  }

  .card.selected {
    transform: translateY(-12px) scale(1.08);
    box-shadow: 
      0 0 0 2px var(--accent),
      0 4px 12px rgba(0, 0, 0, 0.12),
      0 8px 24px rgba(0, 0, 0, 0.08);
    z-index: 10;
  }

  .card.playable:active {
    transform: translateY(-6px) scale(1.03);
  }
</style>
