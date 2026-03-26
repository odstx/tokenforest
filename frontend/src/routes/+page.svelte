<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { auth } from '$lib/auth';

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

  let name = '';
  let symbol = '';
  let supply = 0;

  onMount(async () => {
    if (browser) {
      const token = localStorage.getItem('token');
      if (!token) {
        goto('/login');
        return;
      }
    }
    await fetchTokens();
  });

  async function fetchTokens() {
    try {
      const token = browser ? localStorage.getItem('token') : null;
      const response = await fetch('/api/tokens', {
        headers: {
          ...(token ? { 'Authorization': `Bearer ${token}` } : {})
        }
      });
      if (response.status === 401) {
        auth.logout();
        goto('/login');
        return;
      }
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
      const token = browser ? localStorage.getItem('token') : null;
      const response = await fetch('/api/tokens', {
        method: 'POST',
        headers: { 
          'Content-Type': 'application/json',
          ...(token ? { 'Authorization': `Bearer ${token}` } : {})
        },
        body: JSON.stringify({ name, symbol, supply })
      });
      
      if (response.status === 401) {
        auth.logout();
        goto('/login');
        return;
      }
      if (!response.ok) throw new Error('Failed to create token');
      
      const newToken = await response.json();
      tokens = [newToken, ...tokens];
      
      name = '';
      symbol = '';
      supply = 0;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to create token';
    }
  }
</script>

<div class="p-4 max-w-6xl mx-auto">
  <div class="mb-6">
    <h1 class="text-3xl font-bold">🌲 TokenForest</h1>
    <p class="text-base-content/70">Manage your tokens in the forest</p>
  </div>

  <div class="card bg-base-100 shadow-xl mb-6">
    <div class="card-body">
      <h2 class="card-title">Create New Token</h2>
      <form on:submit|preventDefault={createToken} class="flex flex-col md:flex-row gap-4">
        <div class="form-control flex-1">
          <label class="label" for="name">
            <span class="label-text">Token Name</span>
          </label>
          <input 
            type="text" 
            id="name" 
            class="input input-bordered w-full"
            bind:value={name} 
            placeholder="e.g., Forest Coin"
            required
          />
        </div>
        
        <div class="form-control flex-1">
          <label class="label" for="symbol">
            <span class="label-text">Symbol</span>
          </label>
          <input 
            type="text" 
            id="symbol" 
            class="input input-bordered w-full"
            bind:value={symbol} 
            placeholder="e.g., FST"
            required
          />
        </div>
        
        <div class="form-control flex-1">
          <label class="label" for="supply">
            <span class="label-text">Supply</span>
          </label>
          <input 
            type="number" 
            id="supply" 
            class="input input-bordered w-full"
            bind:value={supply} 
            placeholder="1000000"
            required
          />
        </div>
        
        <div class="form-control md:self-end">
          <button type="submit" class="btn btn-primary" disabled={loading}>
            Create Token 🌱
          </button>
        </div>
      </form>
    </div>
  </div>

  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <h2 class="card-title">All Tokens ({tokens.length})</h2>
      
      {#if loading}
        <div class="text-center py-8">
          <span class="loading loading-spinner loading-lg"></span>
        </div>
      {:else if error}
        <div class="alert alert-error">
          <span>Error: {error}</span>
        </div>
      {:else if tokens.length === 0}
        <div class="text-center py-8 text-base-content/70">
          No tokens yet. Create the first one! 🌱
        </div>
      {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {#each tokens as token (token.id)}
            <div class="card bg-base-200">
              <div class="card-body">
                <h3 class="card-title">{token.name}</h3>
                <div class="badge badge-outline">{token.symbol}</div>
                <p class="text-sm text-base-content/70">Supply: {token.supply.toLocaleString()}</p>
                <p class="text-sm text-base-content/70">Created: {new Date(token.created_at).toLocaleDateString()}</p>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
