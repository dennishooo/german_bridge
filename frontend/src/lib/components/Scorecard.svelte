<script lang="ts">
  import type { PlayerId } from '../stores/websocket';

  // Defines the shape of a single round's result
  interface RoundResult {
      round_number: number;
      bids: Record<string, number>;
      tricks_won: Record<string, number>;
      scores: Record<string, number>;
  }

  export let history: RoundResult[] = [];
  export let players: PlayerId[] = [];
  export let myPlayerId: string;
  export let playerUsernames: Record<string, string> = {};

  function getPlayerName(id: string) {
      if (id === myPlayerId) return "You";
      // Use the provided username mapping, fallback to abbreviated ID
      return playerUsernames[id] || `P${id.slice(0, 4)}`;
  }
  
  // Calculate totals for the footer
  $: totals = players.reduce((acc, pid) => {
      acc[pid] = history.reduce((playerTotals, round) => {
          const bid = round.bids[pid] || 0;
          const made = round.tricks_won[pid] || 0;
          
          playerTotals.bids += bid;
          playerTotals.made += made;
          playerTotals.diff += (bid - made);
          playerTotals.score += (round.scores[pid] || 0);
          return playerTotals;
      }, { bids: 0, made: 0, diff: 0, score: 0 });
      return acc;
  }, {} as Record<string, { bids: number, made: number, diff: number, score: number }>);
</script>

<div class="scorecard">
  <h3>Game History</h3>
  <div class="table-container">
      <table>
          <thead>
              <tr>
                  <th rowspan="2" class="sticky-col">Round</th>
                  {#each players as pid}
                      <th colspan="4" class="player-header">{getPlayerName(pid)}</th>
                  {/each}
              </tr>
              <tr>
                  {#each players as pid}
                      <th class="sub-header">Bid</th>
                      <th class="sub-header">Made</th>
                      <th class="sub-header">Diff</th>
                      <th class="sub-header">Score</th>
                  {/each}
              </tr>
          </thead>
          <tbody>
              {#each history as round}
                  <tr>
                      <td class="round-num sticky-col">{round.round_number}</td>
                      {#each players as pid}
                          <td class="val">{round.bids[pid] ?? '-'}</td>
                          <td class="val" class:made={round.tricks_won[pid] === round.bids[pid]} class:missed={round.tricks_won[pid] !== round.bids[pid]}>
                              {round.tricks_won[pid] ?? 0}
                          </td>
                          <td class="val diff-val">
                              {(round.bids[pid] ?? 0) - (round.tricks_won[pid] ?? 0)}
                          </td>
                          <td class="val score" class:positive={round.scores[pid] > 0} class:negative={round.scores[pid] < 0}>
                              {round.scores[pid] ?? 0}
                          </td>
                      {/each}
                  </tr>
              {/each}
          </tbody>
          <tfoot>
              <tr>
                  <td class="sticky-col total-label">Total</td>
                  {#each players as pid}
                      <td class="val footer-val">{totals[pid].bids}</td>
                      <td class="val footer-val">{totals[pid].made}</td>
                      <td class="val footer-val diff-val">{totals[pid].diff}</td>
                      <td class="val total-score">{totals[pid].score}</td>
                  {/each}
              </tr>
          </tfoot>
      </table>
  </div>
</div>

<style>
  .scorecard {
      background: var(--bg-secondary);
      border-radius: var(--radius-lg);
      padding: var(--spacing-md);
      box-shadow: var(--shadow-lg);
      max-width: 95vw;
      border: 1px solid var(--border-color);
      display: flex;
      flex-direction: column;
      max-height: 80vh;
  }

  h3 {
      margin-top: 0;
      text-align: center;
      margin-bottom: var(--spacing-md);
      flex-shrink: 0;
  }

  .table-container {
      overflow-x: auto;
      overflow-y: auto;
      flex: 1;
      border-radius: var(--radius-md);
      border: 1px solid var(--border-color);
  }

  table {
      width: 100%;
      border-collapse: separate; /* Required for sticky headers/cols */
      border-spacing: 0;
      font-size: 0.9rem;
      min-width: max-content;
  }

  th, td {
      padding: var(--spacing-xs) var(--spacing-sm);
      text-align: center;
      border-right: 1px solid var(--border-color);
      border-bottom: 1px solid var(--border-color);
      background: var(--bg-secondary); /* Needed for sticky to cover content */
  }

  /* Sticky Headers */
  thead {
      position: sticky;
      top: 0;
      z-index: 10;
  }
  
  th {
      background: var(--bg-tertiary);
      font-weight: bold;
      color: var(--text-primary);
  }

  .player-header {
      border-bottom: 2px solid var(--border-color);
  }

  .sub-header {
      font-size: 0.8rem;
      font-weight: normal;
      color: var(--text-secondary);
  }

  /* Sticky First Column */
  .sticky-col {
      position: sticky;
      left: 0;
      z-index: 5; /* Lower than header */
      background: var(--bg-tertiary);
      border-right: 2px solid var(--border-color);
  }
  
  thead tr:first-child th.sticky-col {
      z-index: 20; /* Highest for top-left intersection */
  }

  .round-num {
      font-weight: bold;
      color: var(--text-primary);
  }

  .val {
      min-width: 30px;
  }

  /* Highlights */
  .made {
      color: var(--color-success);
      font-weight: bold;
  }
  
  .missed {
      color: var(--color-error);
  }

  .diff-val {
      color: var(--text-tertiary);
      font-size: 0.9em;
  }

  .score {
      font-weight: bold;
      background: rgba(0,0,0,0.05); /* Slight tint for score column */
  }

  .score.positive { color: var(--color-success); }
  .score.negative { color: var(--color-error); }

  /* Footer */
  tfoot {
      position: sticky;
      bottom: 0;
      z-index: 10;
  }

  tfoot td {
      background: var(--bg-tertiary);
      border-top: 2px solid var(--border-color);
      font-weight: bold;
  }

  .total-label {
      text-align: right;
      padding-right: var(--spacing-md);
  }

  .total-score {
      font-size: 1rem;
      color: var(--color-warning);
  }

  .footer-val {
      font-weight: bold;
      color: var(--text-secondary);
  }
</style>
