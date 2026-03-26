<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '../../lib/auth';

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
        error = data.error || 'Login failed';
        return;
      }

      auth.login(data.token, data.user);
      goto('/');
    } catch (err) {
      error = err instanceof Error ? err.message : 'Login failed';
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex items-center justify-center min-h-screen">
  <div class="card w-full max-w-sm bg-base-100 shadow-xl">
    <div class="card-body">
      <h1 class="card-title text-2xl justify-center">🌲 TokenForest</h1>
      <p class="text-center text-base-content/70 mb-4">Sign in to your account</p>

      <form on:submit|preventDefault={handleLogin}>
        {#if error}
          <div class="alert alert-error mb-4">
            <span>{error}</span>
          </div>
        {/if}

        <div class="form-control w-full mb-4">
          <label class="label" for="username">
            <span class="label-text">Username</span>
          </label>
          <input
            type="text"
            id="username"
            class="input input-bordered w-full"
            bind:value={username}
            placeholder="Enter your username"
            required
          />
        </div>

        <div class="form-control w-full mb-4">
          <label class="label" for="password">
            <span class="label-text">Password</span>
          </label>
          <input
            type="password"
            id="password"
            class="input input-bordered w-full"
            bind:value={password}
            placeholder="Enter your password"
            required
          />
        </div>

        <div class="form-control mt-6">
          <button type="submit" class="btn btn-primary" disabled={loading}>
            {loading ? 'Signing in...' : 'Sign In'}
          </button>
        </div>

        <p class="text-center mt-4">
          Don't have an account? <a href="/register" class="link link-primary">Register</a>
        </p>
      </form>
    </div>
  </div>
</div>
