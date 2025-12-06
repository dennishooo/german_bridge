<script lang="ts">
  export let value = '';
  export let placeholder = '';
  export let type: 'text' | 'email' | 'password' | 'number' = 'text';
  export let disabled = false;
  export let label = '';
</script>

<div class="input-wrapper">
  {#if label}
    <label class="input-label">{label}</label>
  {/if}
  <input
    class="input"
    {type}
    {placeholder}
    {disabled}
    bind:value
    on:input
    on:change
    on:focus
    on:blur
  />
</div>

<style>
  .input-wrapper {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }
  
  .input-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
  }
  
  .input {
    padding: 0.5rem 0.75rem;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: 1rem;
    transition: all 0.2s ease;
  }
  
  .input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 10%, transparent);
  }
  
  .input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .input::placeholder {
    color: var(--text-tertiary);
  }
</style>
