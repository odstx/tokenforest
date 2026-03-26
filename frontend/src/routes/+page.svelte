<script lang="ts">
  import { onMount } from 'svelte';

  interface Token {
    id: number;
    name: string;
    symbol: string;
    supply: number;
    created_at: string;
  }

  let tokens: Token[] = [];
  let loading = true;
  let error: string | null = null;

  // Form state
  let name = '';
  let symbol = '';
  let supply = 0;

  onMount(async () => {
    await fetchTokens();
  });

  async function fetchTokens() {
    try {
      const response = await fetch('/api/tokens');
      if (!response.ok) throw new Error('Failed to fetch tokens');
      tokens = await response.json();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unknown error';
    } finally {
      loading = false;
    }
  }

  async function createToken() {
    try {
      const response = await fetch('/api/tokens', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, symbol, supply })
      });
      
      if (!response.ok) throw new Error('Failed to create token');
      
      const newToken = await response.json();
      tokens = [newToken, ...tokens];
      
      // Reset form
      name = '';
      symbol = '';
      supply = 0;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to create token';
    }
  }
</script>

<main>
  <h1>🌲 TokenForest</h1>
  <p class="subtitle">Manage your tokens in the forest</p>

  <section class="create-token">
    <h2>Create New Token</h2>
    <form on:submit|preventDefault={createToken}>
      <div class="form-group">
        <label for="name">Token Name</label>
        <input 
          type="text" 
          id="name" 
          bind:value={name} 
          placeholder="e.g., Forest Coin"
          required
        />
      </div>
      
      <div class="form-group">
        <label for="symbol">Symbol</label>
        <input 
          type="text" 
          id="symbol" 
          bind:value={symbol} 
          placeholder="e.g., FST"
          required
        />
      </div>
      
      <div class="form-group">
        <label for="supply">Supply</label>
        <input 
          type="number" 
          id="supply" 
          bind:value={supply} 
          placeholder="1000000"
          required
        />
      </div>
      
      <button type="submit" disabled={loading}>Create Token 🌱</button>
    </form>
  </section>

  <section class="token-list">
    <h2>All Tokens ({tokens.length})</h2>
    
    {#if loading}
      <p class="loading">Loading tokens...</p>
    {:else if error}
      <p class="error">Error: {error}</p>
    {:else if tokens.length === 0}
      <p class="empty">No tokens yet. Create the first one! 🌱</p>
    {:else}
      <div class="tokens-grid">
        {#each tokens as token (token.id)}
          <div class="token-card">
            <h3>{token.name}</h3>
            <span class="symbol">{token.symbol}</span>
            <p class="supply">Supply: {token.supply.toLocaleString()}</p>
            <p class="created">Created: {new Date(token.created_at).toLocaleDateString()}</p>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
  }

  main {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
    color: white;
  }

  h1 {
    font-size: 3rem;
    margin-bottom: 0.5rem;
    text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
  }

  .subtitle {
    font-size: 1.2rem;
    opacity: 0.9;
    margin-bottom: 2rem;
  }

  section {
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(10px);
    border-radius: 16px;
    padding: 2rem;
    margin-bottom: 2rem;
  }

  h2 {
    margin-top: 0;
    border-bottom: 2px solid rgba(255,255,255,0.3);
    padding-bottom: 0.5rem;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
  }

  input {
    width: 100%;
    padding: 0.75rem;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    box-sizing: border-box;
  }

  button {
    background: #4CAF50;
    color: white;
    border: none;
    padding: 1rem 2rem;
    font-size: 1rem;
    border-radius: 8px;
    cursor: pointer;
    transition: transform 0.2s, background 0.2s;
  }

  button:hover:not(:disabled) {
    background: #45a049;
    transform: translateY(-2px);
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .tokens-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1.5rem;
  }

  .token-card {
    background: rgba(255, 255, 255, 0.15);
    padding: 1.5rem;
    border-radius: 12px;
    transition: transform 0.2s;
  }

  .token-card:hover {
    transform: translateY(-4px);
    background: rgba(255, 255, 255, 0.2);
  }

  .token-card h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.5rem;
  }

  .symbol {
    display: inline-block;
    background: rgba(255, 255, 255, 0.2);
    padding: 0.25rem 0.75rem;
    border-radius: 20px;
    font-weight: bold;
    margin-bottom: 1rem;
  }

  .supply, .created {
    margin: 0.5rem 0;
    opacity: 0.9;
  }

  .loading, .error, .empty {
    text-align: center;
    padding: 2rem;
    font-size: 1.2rem;
  }

  .error {
    color: #ff6b6b;
    background: rgba(255, 107, 107, 0.2);
    border-radius: 8px;
  }
</style>
