<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '../../lib/auth';
  import { _ } from 'svelte-i18n';

  let username = '';
  let password = '';
  let error: string | null = null;
  let loading = false;

  async function handleLogin() {
    error = null;
    loading = true;

    try {
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password })
      });

      const data = await response.json();

      if (!response.ok) {
        error = data.error || $_('errors.loginFailed');
        return;
      }

      auth.login(data.token, data.user);
      goto('/');
    } catch (err) {
      error = err instanceof Error ? err.message : $_('errors.loginFailed');
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex items-center justify-center min-h-screen">
  <div class="card w-full max-w-sm bg-base-100 shadow-xl">
    <div class="card-body">
      <h1 class="card-title text-2xl justify-center">🌲 {$_('app.name')}</h1>
      <p class="text-center text-base-content/70 mb-4">{$_('login.title')}</p>

      <form on:submit|preventDefault={handleLogin}>
        {#if error}
          <div class="alert alert-error mb-4">
            <span>{error}</span>
          </div>
        {/if}

        <div class="form-control w-full mb-4">
          <label class="label" for="username">
            <span class="label-text">{$_('login.username')}</span>
          </label>
          <input
            type="text"
            id="username"
            class="input input-bordered w-full"
            bind:value={username}
            placeholder={$_('login.usernamePlaceholder')}
            required
          />
        </div>

        <div class="form-control w-full mb-4">
          <label class="label" for="password">
            <span class="label-text">{$_('login.password')}</span>
          </label>
          <input
            type="password"
            id="password"
            class="input input-bordered w-full"
            bind:value={password}
            placeholder={$_('login.passwordPlaceholder')}
            required
          />
        </div>

        <div class="form-control mt-6">
          <button type="submit" class="btn btn-primary" disabled={loading}>
            {loading ? $_('login.signingIn') : $_('login.signIn')}
          </button>
        </div>

        <p class="text-center mt-4">
          {$_('login.noAccount')} <a href="/register" class="link link-primary">{$_('login.register')}</a>
        </p>
      </form>
    </div>
  </div>
</div>
