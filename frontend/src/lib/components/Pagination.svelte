<script lang="ts">
  export let page: number;
  export let totalPages: number;
  export let onPageChange: (p: number) => void;

  function goToPage(p: number) {
    if (p < 1 || p > totalPages) return;
    onPageChange(p);
  }
</script>

{#if totalPages > 1}
  <div class="flex justify-center mt-4 join">
    <button 
      class="join-item btn" 
      on:click={() => goToPage(page - 1)}
      disabled={page <= 1}
    >
      «
    </button>
    {#each Array.from({ length: totalPages }, (_, i) => i + 1) as p}
      <button 
        class="join-item btn {p === page ? 'btn-active' : ''}" 
        on:click={() => goToPage(p)}
      >
        {p}
      </button>
    {/each}
    <button 
      class="join-item btn" 
      on:click={() => goToPage(page + 1)}
      disabled={page >= totalPages}
    >
      »
    </button>
  </div>
{/if}
