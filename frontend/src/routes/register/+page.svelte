<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '../../lib/auth';

  let username = '';
  let password = '';
  let confirmPassword = '';
  let error: string | null = null;
  let loading = false;

  async function handleRegister() {
    error = null;

    if (password !== confirmPassword) {
      error = 'Passwords do not match';
      return;
    }

    if (password.length < 6) {
      error = 'Password must be at least 6 characters';
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
        error = data.error || 'Registration failed';
        return;
      }

      auth.login(data.token, data.user);
      goto('/');
    } catch (err) {
      error = err instanceof Error ? err.message : 'Registration failed';
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex items-center justify-center min-h-screen">
  <div class="card w-full max-w-sm bg-base-100 shadow-xl">
    <div class="card-body">
      <h1 class="card-title text-2xl justify-center">🌲 TokenForest</h1>
      <p class="text-center text-base-content/70 mb-4">Create your account</p>

      <form on:submit|preventDefault={handleRegister}>
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
            placeholder="Choose a username"
            required
            minlength={3}
            maxlength={50}
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
            placeholder="Create a password"
            required
            minlength={6}
          />
        </div>

        <div class="form-control w-full mb-4">
          <label class="label" for="confirmPassword">
            <span class="label-text">Confirm Password</span>
          </label>
          <input
            type="password"
            id="confirmPassword"
            class="input input-bordered w-full"
            bind:value={confirmPassword}
            placeholder="Confirm your password"
            required
          />
        </div>

        <div class="form-control mt-6">
          <button type="submit" class="btn btn-primary" disabled={loading}>
            {loading ? 'Creating account...' : 'Create Account'}
          </button>
        </div>

        <p class="text-center mt-4">
          Already have an account? <a href="/login" class="link link-primary">Sign in</a>
        </p>
      </form>
    </div>
  </div>
</div>
