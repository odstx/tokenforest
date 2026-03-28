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
          <a href="/api-keys" class="btn btn-ghost btn-sm">{$_('nav.apiKeys')}</a>
          <span class="text-sm">{$_('app.welcome', { values: { username: $auth.user?.username } })}</span>
          <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-ghost btn-sm">
              {$currentLocale === 'zh' ? '中文' : 'EN'}
            </label>
            <ul tabindex="0" class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-24">
              <li><button on:click={() => switchLanguage('en')}>English</button></li>
              <li><button on:click={() => switchLanguage('zh')}>中文</button></li>
            </ul>
          </div>
          <button class="btn btn-outline btn-sm" on:click={handleLogout}>{$_('nav.logout')}</button>
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
            <ul tabindex="0" class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-24">
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
