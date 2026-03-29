<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';

  interface Stats {
    api_keys_count: number;
    token_pools_count: number;
  }

  let stats: Stats = { api_keys_count: 0, token_pools_count: 0 };
  let loading = true;

  async function fetchStats() {
    if (!browser) return;
    const token = localStorage.getItem('token');
    if (!token) {
      goto('/login');
      return;
    }

    try {
      const res = await fetch('/api/stats', {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });

      if (res.ok) {
        stats = await res.json();
      } else if (res.status === 401) {
        localStorage.removeItem('token');
        goto('/login');
      }
    } catch (e) {
      console.error('Failed to fetch stats:', e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchStats();
  });
</script>

<div class="p-4 max-w-6xl mx-auto">
  {#if loading}
    <div class="flex justify-center items-center h-64">
      <span class="loading loading-spinner loading-lg"></span>
    </div>
  {:else}
    <h1 class="text-2xl font-bold mb-6">Dashboard</h1>
    
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <a href="/api-keys" class="card bg-base-200 shadow hover:bg-base-300 transition-colors cursor-pointer">
        <div class="card-body">
          <h2 class="card-title text-lg">API Keys</h2>
          <p class="text-4xl font-bold text-primary">{stats.api_keys_count}</p>
        </div>
      </a>
      
      <a href="/token-pools" class="card bg-base-200 shadow hover:bg-base-300 transition-colors cursor-pointer">
        <div class="card-body">
          <h2 class="card-title text-lg">Token Pools</h2>
          <p class="text-4xl font-bold text-secondary">{stats.token_pools_count}</p>
        </div>
      </a>
    </div>
  {/if}
</div>
