<script lang="ts">
  import ThemeToggle from '$lib/components/ThemeToggle.svelte';
  import Button from '$lib/components/Button.svelte';
  import Container from '$lib/components/Container.svelte';
  import Input from '$lib/components/Input.svelte';
  
  let inputValue = '';
</script>

<div class="page">
  <header class="header">
    <h1 class="title">Design System</h1>
    <ThemeToggle />
  </header>
  
  <div class="content">
    <!-- Colors -->
    <section class="section">
      <h2 class="section-title">Colors</h2>
      <Container>
        <div class="color-grid">
          <div class="color-item">
            <div class="color-swatch" style="background-color: var(--accent)"></div>
            <span class="color-label">Accent</span>
          </div>
          <div class="color-item">
            <div class="color-swatch" style="background-color: var(--color-success)"></div>
            <span class="color-label">Success</span>
          </div>
          <div class="color-item">
            <div class="color-swatch" style="background-color: var(--color-warning)"></div>
            <span class="color-label">Warning</span>
          </div>
          <div class="color-item">
            <div class="color-swatch" style="background-color: var(--color-error)"></div>
            <span class="color-label">Error</span>
          </div>
        </div>
      </Container>
    </section>
    
    <!-- Typography -->
    <section class="section">
      <h2 class="section-title">Typography</h2>
      <Container>
        <h1 style="margin-bottom: 0.5rem;">Heading 1</h1>
        <h2 style="margin-bottom: 0.5rem;">Heading 2</h2>
        <h3 style="margin-bottom: 0.5rem;">Heading 3</h3>
        <p class="text-primary">Primary text - The quick brown fox jumps over the lazy dog</p>
        <p class="text-secondary">Secondary text - The quick brown fox jumps over the lazy dog</p>
        <p class="text-tertiary">Tertiary text - The quick brown fox jumps over the lazy dog</p>
      </Container>
    </section>
    
    <!-- Buttons -->
    <section class="section">
      <h2 class="section-title">Buttons</h2>
      <Container>
        <div class="component-grid">
          <div>
            <p class="component-label">Primary</p>
            <Button variant="primary">Button</Button>
          </div>
          <div>
            <p class="component-label">Secondary</p>
            <Button variant="secondary">Button</Button>
          </div>
          <div>
            <p class="component-label">Outline</p>
            <Button variant="outline">Button</Button>
          </div>
          <div>
            <p class="component-label">Ghost</p>
            <Button variant="ghost">Button</Button>
          </div>
        </div>
        
        <div class="component-grid" style="margin-top: 1rem;">
          <div>
            <p class="component-label">Small</p>
            <Button size="sm">Button</Button>
          </div>
          <div>
            <p class="component-label">Medium</p>
            <Button size="md">Button</Button>
          </div>
          <div>
            <p class="component-label">Large</p>
            <Button size="lg">Button</Button>
          </div>
          <div>
            <p class="component-label">Disabled</p>
            <Button disabled>Button</Button>
          </div>
        </div>
      </Container>
    </section>
    
    <!-- Inputs -->
    <section class="section">
      <h2 class="section-title">Inputs</h2>
      <Container>
        <div style="display: flex; flex-direction: column; gap: 1rem; max-width: 400px;">
          <Input label="Email" type="email" placeholder="Enter your email" bind:value={inputValue} />
          <Input label="Password" type="password" placeholder="Enter your password" />
          <Input label="Disabled" disabled placeholder="Disabled input" />
        </div>
      </Container>
    </section>
    
    <!-- Containers -->
    <section class="section">
      <h2 class="section-title">Containers</h2>
      <div class="component-grid">
        <Container padding="sm" shadow="sm">
          <h3>Small Container</h3>
          <p class="text-secondary">Small padding & shadow</p>
        </Container>
        <Container padding="md" shadow="md">
          <h3>Medium Container</h3>
          <p class="text-secondary">Medium padding & shadow</p>
        </Container>
        <Container padding="lg" shadow="lg">
          <h3>Large Container</h3>
          <p class="text-secondary">Large padding & shadow</p>
        </Container>
      </div>
    </section>
  </div>
</div>

<style>
  .page {
    min-height: 100vh;
    padding: var(--spacing-xl);
  }
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-2xl);
  }
  
  .title {
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }
  
  .content {
    max-width: 1200px;
    margin: 0 auto;
  }
  
  .section {
    margin-bottom: var(--spacing-2xl);
  }
  
  .section-title {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--spacing-md);
  }
  
  .color-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: var(--spacing-md);
  }
  
  .color-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-sm);
  }
  
  .color-swatch {
    width: 80px;
    height: 80px;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
  }
  
  .color-label {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }
  
  .component-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: var(--spacing-lg);
  }
  
  .component-label {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: var(--spacing-sm);
  }
  
  .text-primary {
    color: var(--text-primary);
  }
  
  .text-secondary {
    color: var(--text-secondary);
  }
  
  .text-tertiary {
    color: var(--text-tertiary);
  }
</style>
