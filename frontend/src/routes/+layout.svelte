<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { auth } from '../lib/auth';

  let mounted = false;

  onMount(() => {
    mounted = true;
  });

  function handleLogout() {
    auth.logout();
    goto('/login');
  }

  $: currentPath = $page.url.pathname;
  $: isAuthenticated = $auth.token !== null;
  $: isAuthPage = currentPath.startsWith('/login') || currentPath.startsWith('/register');
</script>

{#if mounted}
  <div class="min-h-screen bg-base-200">
    {#if isAuthenticated && !isAuthPage}
      <div class="navbar bg-base-100 shadow-sm">
        <div class="flex-1">
          <a href="/" class="btn btn-ghost text-xl">🌲 TokenForest</a>
        </div>
        <div class="flex-none gap-2">
          <span class="text-sm">Welcome, {$auth.user?.username}</span>
          <button class="btn btn-outline btn-sm" on:click={handleLogout}>Logout</button>
        </div>
      </div>
    {/if}
    <slot />
  </div>
{/if}
