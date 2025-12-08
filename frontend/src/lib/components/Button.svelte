<script lang="ts">
const {
  variant = 'primary',
  size = 'md',
  disabled = false,
  type = 'button',
  fullWidth = false,
  onclick,
  children = null
} = $props<{
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  type?: 'button' | 'submit' | 'reset';
  fullWidth?: boolean;
  onclick?: (e: MouseEvent) => void;
  children?: import('svelte').Snippet | null;
}>();
</script>

<button
  class="btn btn-{variant} btn-{size}"
  class:full-width={fullWidth}
  {type}
  {disabled}
  {onclick}
>
  {#if children}
    {@render children()}
  {/if}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-weight: 500;
    border-radius: var(--radius-md);
    transition: all 0.2s ease;
    cursor: pointer;
    border: 1px solid transparent;
  }
  
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  /* Sizes */
  .btn-sm {
    padding: 0.375rem 0.75rem;
    font-size: 0.875rem;
  }
  
  .btn-md {
    padding: 0.5rem 1rem;
    font-size: 1rem;
  }
  
  .btn-lg {
    padding: 0.75rem 1.5rem;
    font-size: 1.125rem;
  }
  
  /* Variants */
  .btn-primary {
    background-color: var(--accent);
    color: white;
  }
  
  .btn-primary:hover:not(:disabled) {
    background-color: var(--accent-hover);
  }
  
  .btn-secondary {
    background-color: var(--bg-secondary);
    color: var(--text-primary);
    border-color: var(--border-color);
  }
  
  .btn-secondary:hover:not(:disabled) {
    background-color: var(--bg-tertiary);
  }
  
  .btn-outline {
    background-color: transparent;
    color: var(--accent);
    border-color: var(--accent);
  }
  
  .btn-outline:hover:not(:disabled) {
    background-color: var(--accent);
    color: white;
  }
  
  .btn-ghost {
    background-color: transparent;
    color: var(--text-primary);
  }
  
  .btn-ghost:hover:not(:disabled) {
    background-color: var(--bg-tertiary);
  }

  .full-width {
    width: 100%;
    display: flex;
  }
</style>
