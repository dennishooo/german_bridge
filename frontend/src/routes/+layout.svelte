<script lang="ts">
  import { onMount } from 'svelte';
  import { theme } from '$lib/stores/theme';
  import '../app.css';
  
  const { children } = $props<{ children: import('svelte').Snippet }>();
  
  onMount(() => {
    theme.init();
  });
</script>

{@render children()}
