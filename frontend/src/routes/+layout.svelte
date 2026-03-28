<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { auth } from '../lib/auth';
  import { setupI18n, setLocale, currentLocale } from '../lib/i18n';
  import { _ } from 'svelte-i18n';

  let mounted = false;

  onMount(() => {
    setupI18n();
    mounted = true;
  });

  function handleLogout() {
    auth.logout();
    goto('/login');
  }

  function switchLanguage(lang: string) {
    setLocale(lang);
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
          <a href="/" class="btn btn-ghost text-xl">🌲 {$_('app.name')}</a>
        </div>
        <div class="flex-none gap-2">
          <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-sm gap-1">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" />
              </svg>
              {$auth.user?.username}
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
              </svg>
            </label>
            <ul tabindex="0" class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-40 z-[9999]">
              <li><a href="/api-keys">{$_('nav.apiKeys')}</a></li>
              <li><a href="/token-pools">{$_('nav.tokenPools')}</a></li>
              <li class="border-t border-base-300 mt-1 pt-1"><button on:click={handleLogout}>{$_('nav.logout')}</button></li>
            </ul>
          </div>
          <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-sm">
              {$currentLocale === 'zh' ? '中文' : 'EN'}
            </label>
            <ul tabindex="0" class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-24 z-[9999]">
              <li><button on:click={() => switchLanguage('en')}>English</button></li>
              <li><button on:click={() => switchLanguage('zh')}>中文</button></li>
            </ul>
          </div>
        </div>
      </div>
    {:else if !isAuthPage}
      <div class="navbar bg-base-100 shadow-sm">
        <div class="flex-1">
          <a href="/" class="btn btn-ghost text-xl">🌲 {$_('app.name')}</a>
        </div>
        <div class="flex-none gap-2">
          <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-sm">
              {$currentLocale === 'zh' ? '中文' : 'EN'}
            </label>
            <ul tabindex="0" class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-24 z-[9999]">
              <li><button on:click={() => switchLanguage('en')}>English</button></li>
              <li><button on:click={() => switchLanguage('zh')}>中文</button></li>
            </ul>
          </div>
        </div>
      </div>
    {/if}
    <slot />
  </div>
{/if}
