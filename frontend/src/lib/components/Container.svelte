<script lang="ts">
  const { padding = 'md', shadow = 'md', children } = $props<{ padding?: 'sm' | 'md' | 'lg'; shadow?: 'sm' | 'md' | 'lg'; children: import('svelte').Snippet }>();
</script>

<div class="container container-padding-{padding} container-shadow-{shadow}">
  {@render children()}
</div>

<style>
  .container {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    transition: all 0.2s ease;
  }
  
  .container-padding-sm {
    padding: var(--spacing-sm);
  }
  
  .container-padding-md {
    padding: var(--spacing-md);
  }
  
  .container-padding-lg {
    padding: var(--spacing-lg);
  }
  
  .container-shadow-sm {
    box-shadow: var(--shadow-sm);
  }
  
  .container-shadow-md {
    box-shadow: var(--shadow-md);
  }
  
  .container-shadow-lg {
    box-shadow: var(--shadow-lg);
  }
</style>
