<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '../../lib/auth';
  import { _ } from 'svelte-i18n';

  let username = '';
  let password = '';
  let confirmPassword = '';
  let error: string | null = null;
  let loading = false;

  async function handleRegister() {
    error = null;

    if (password !== confirmPassword) {
      error = $_('register.passwordMismatch');
      return;
    }

    if (password.length < 6) {
      error = $_('register.passwordTooShort');
      return;
    }

    loading = true;

    try {
      const response = await fetch('/api/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password })
      });

      const data = await response.json();

      if (!response.ok) {
        error = data.error || $_('errors.registrationFailed');
        return;
      }

      auth.login(data.token, data.user);
      goto('/');
    } catch (err) {
      error = err instanceof Error ? err.message : $_('errors.registrationFailed');
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex items-center justify-center min-h-screen">
  <div class="card w-full max-w-sm bg-base-100 shadow-xl">
    <div class="card-body">
      <h1 class="card-title text-2xl justify-center">🌲 {$_('app.name')}</h1>
      <p class="text-center text-base-content/70 mb-4">{$_('register.title')}</p>

      <form on:submit|preventDefault={handleRegister}>
        {#if error}
          <div class="alert alert-error mb-4">
            <span>{error}</span>
          </div>
        {/if}

        <div class="form-control w-full mb-4">
          <label class="label" for="username">
            <span class="label-text">{$_('register.username')}</span>
          </label>
          <input
            type="text"
            id="username"
            class="input input-bordered w-full"
            bind:value={username}
            placeholder={$_('register.usernamePlaceholder')}
            required
            minlength={3}
            maxlength={50}
          />
        </div>

        <div class="form-control w-full mb-4">
          <label class="label" for="password">
            <span class="label-text">{$_('register.password')}</span>
          </label>
          <input
            type="password"
            id="password"
            class="input input-bordered w-full"
            bind:value={password}
            placeholder={$_('register.passwordPlaceholder')}
            required
            minlength={6}
          />
        </div>

        <div class="form-control w-full mb-4">
          <label class="label" for="confirmPassword">
            <span class="label-text">{$_('register.confirmPassword')}</span>
          </label>
          <input
            type="password"
            id="confirmPassword"
            class="input input-bordered w-full"
            bind:value={confirmPassword}
            placeholder={$_('register.confirmPasswordPlaceholder')}
            required
          />
        </div>

        <div class="form-control mt-6">
          <button type="submit" class="btn btn-primary" disabled={loading}>
            {loading ? $_('register.creating') : $_('register.createAccount')}
          </button>
        </div>

        <p class="text-center mt-4">
          {$_('register.hasAccount')} <a href="/login" class="link link-primary">{$_('register.signIn')}</a>
        </p>
      </form>
    </div>
  </div>
</div>
