<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';

  onMount(async () => {
    if (browser) {
      const token = localStorage.getItem('token');
      if (!token) {
        goto('/login');
      }
    }
  });
</script>

<div class="p-4 max-w-6xl mx-auto">
</div>
