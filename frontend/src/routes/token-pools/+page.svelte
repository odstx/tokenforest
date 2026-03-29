<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { _ } from 'svelte-i18n';
  import { t } from '$lib/i18n';
  import { commonModels, type Model } from '$lib/components/commonModels';
  import ModelDropdown from '$lib/components/ModelDropdown.svelte';
  import Pagination from '$lib/components/Pagination.svelte';

  interface TokenPool {
    id: number;
    name: string;
    model_type: string;
    base_url: string;
    is_active: boolean;
    last_used_at: string | null;
    created_at: string;
    updated_at: string;
  }

  interface PaginatedResponse {
    items: TokenPool[];
    total: number;
    page: number;
    page_size: number;
    total_pages: number;
  }

  let tokenPools: TokenPool[] = [];
  let loading = true;
  let error: string | null = null;
  let showModal = false;
  let editMode = false;
  let editingId: number | null = null;
  let formData = {
    name: '',
    model_type: '',
    base_url: '',
    api_key: ''
  };
  let saving = false;
  let testingId: number | null = null;
  let testResults: Map<number, { success: boolean; message: string; responseContent?: string }> = new Map();
  let actionDropdownOpen: number | null = null;
  let actionDropdownTop = 0;
  let actionDropdownLeft = 0;

  function selectModel(model: Model) {
    formData.model_type = model.id;
  }

  function openActionDropdown(event: MouseEvent, poolId: number) {
    const target = event.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    actionDropdownTop = rect.bottom + 4;
    actionDropdownLeft = rect.right - 160;
    actionDropdownOpen = poolId;
  }

  function closeActionDropdown() {
    actionDropdownOpen = null;
  }

  function handleEditAction() {
    const pool = tokenPools.find(p => p.id === actionDropdownOpen);
    if (pool) {
      closeActionDropdown();
      openEditModal(pool);
    }
  }

  function handleTestAction() {
    const poolId = actionDropdownOpen;
    if (poolId !== null) {
      closeActionDropdown();
      testPool(poolId);
    }
  }

  function handleChatAction() {
    const poolId = actionDropdownOpen;
    if (poolId !== null) {
      closeActionDropdown();
      goto(`/chat?type=token-pool&id=${poolId}`);
    }
  }

  function handleToggleAction() {
    const poolId = actionDropdownOpen;
    if (poolId !== null) {
      closeActionDropdown();
      togglePool(poolId);
    }
  }

  function handleDeleteAction() {
    const poolId = actionDropdownOpen;
    if (poolId !== null) {
      closeActionDropdown();
      deletePool(poolId);
    }
  }

  function getActionPool() {
    return tokenPools.find(p => p.id === actionDropdownOpen);
  }

  let page = 1;
  let pageSize = 10;
  let total = 0;
  let totalPages = 0;

  async function fetchTokenPools() {
    if (!browser) return;
    
    const token = localStorage.getItem('token');
    if (!token) {
      goto('/login');
      return;
    }

    try {
      const response = await fetch(`/api/token-pools?page=${page}&page_size=${pageSize}`, {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });

      if (!response.ok) {
        if (response.status === 401) {
          goto('/login');
          return;
        }
        throw new Error('Failed to fetch token pools');
      }

      const data: PaginatedResponse = await response.json();
      tokenPools = data.items || [];
      total = data.total || 0;
      totalPages = data.total_pages || 0;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load token pools';
    } finally {
      loading = false;
    }
  }

  async function goToPage(p: number) {
    if (p < 1 || p > totalPages) return;
    page = p;
    loading = true;
    await fetchTokenPools();
  }

  function openCreateModal() {
    editMode = false;
    editingId = null;
    formData = { name: '', model_type: '', base_url: '', api_key: '' };
    showModal = true;
  }

  function openEditModal(pool: TokenPool) {
    editMode = true;
    editingId = pool.id;
    formData = {
      name: pool.name,
      model_type: pool.model_type,
      base_url: pool.base_url,
      api_key: ''
    };
    showModal = true;
  }

  async function savePool() {
    if (!browser) return;
    if (!formData.name.trim() || !formData.model_type.trim() || !formData.base_url.trim()) return;
    if (!editMode && !formData.api_key.trim()) return;
    
    const token = localStorage.getItem('token');
    if (!token) return;

    saving = true;
    error = null;

    try {
      const url = editMode ? `/api/token-pools/${editingId}` : '/api/token-pools';
      const method = editMode ? 'PUT' : 'POST';
      
      const body: Record<string, string | undefined> = {
        name: formData.name.trim(),
        model_type: formData.model_type.trim(),
        base_url: formData.base_url.trim(),
      };
      
      if (formData.api_key.trim()) {
        body.api_key = formData.api_key.trim();
      }

      const response = await fetch(url, {
        method,
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(body)
      });

      if (!response.ok) {
        throw new Error('Failed to save token pool');
      }

      closeModal();
      await fetchTokenPools();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to save token pool';
    } finally {
      saving = false;
    }
  }

  async function deletePool(id: number) {
    if (!browser) return;
    if (!confirm(t('tokenPools.confirmDelete'))) return;
    
    const token = localStorage.getItem('token');
    if (!token) return;

    try {
      const response = await fetch(`/api/token-pools/${id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });

      if (!response.ok) {
        throw new Error('Failed to delete token pool');
      }

      if (tokenPools.length === 1 && page > 1) {
        page--;
      }
      await fetchTokenPools();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to delete token pool';
    }
  }

  async function togglePool(id: number) {
    if (!browser) return;
    
    const token = localStorage.getItem('token');
    if (!token) return;

    try {
      const response = await fetch(`/api/token-pools/${id}/toggle`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });

      if (!response.ok) {
        throw new Error('Failed to toggle token pool');
      }

      await fetchTokenPools();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to toggle token pool';
    }
  }

  async function testPool(id: number) {
    if (!browser) return;
    
    const token = localStorage.getItem('token');
    if (!token) return;

    testingId = id;
    testResults.delete(id);

    try {
      const response = await fetch(`/api/token-pools/${id}/test`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });

      const data = await response.json();
      testResults.set(id, {
        success: data.success,
        message: data.message,
        responseContent: data.response_content
      });
    } catch (err) {
      testResults.set(id, {
        success: false,
        message: err instanceof Error ? err.message : 'Test failed',
        responseContent: undefined
      });
    } finally {
      testingId = null;
    }
  }

  function closeModal() {
    showModal = false;
    editMode = false;
    editingId = null;
    formData = { name: '', model_type: '', base_url: '', api_key: '' };
  }

  onMount(fetchTokenPools);
</script>

<div class="p-4 max-w-4xl mx-auto">
  <div class="flex justify-between items-center mb-6">
    <h1 class="text-2xl font-bold">{$_('tokenPools.title')}</h1>
    <button class="btn btn-primary" on:click={openCreateModal}>
      {$_('tokenPools.createButton')}
    </button>
  </div>

  {#if error}
    <div class="alert alert-error mb-4">
      <span>{error}</span>
    </div>
  {/if}

  {#if loading}
    <div class="flex justify-center py-8">
      <span class="loading loading-spinner loading-lg"></span>
    </div>
  {:else if tokenPools.length === 0}
    <div class="text-center py-8 text-base-content/70">
      <p>{$_('tokenPools.noPools')}</p>
    </div>
  {:else}
    <div class="overflow-x-auto">
      <table class="table table-zebra w-full">
        <thead>
          <tr>
            <th>{$_('tokenPools.table.name')}</th>
            <th>{$_('tokenPools.table.modelType')}</th>
            <th>{$_('tokenPools.table.baseUrl')}</th>
            <th>{$_('tokenPools.table.status')}</th>
            <th>{$_('tokenPools.table.lastUsed')}</th>
            <th>{$_('tokenPools.table.created')}</th>
            <th>{$_('tokenPools.table.actions')}</th>
          </tr>
        </thead>
        <tbody>
          {#each tokenPools as pool}
            <tr>
              <td>{pool.name}</td>
              <td>
                <span class="badge badge-outline badge-sm">{pool.model_type}</span>
              </td>
              <td><code class="bg-base-300 px-2 py-1 rounded text-xs">{pool.base_url}</code></td>
              <td>
                <span class="badge {pool.is_active ? 'badge-success' : 'badge-error'}">
                  {pool.is_active ? $_('tokenPools.status.active') : $_('tokenPools.status.inactive')}
                </span>
              </td>
              <td>{pool.last_used_at || $_('tokenPools.lastUsed.never')}</td>
              <td>{new Date(pool.created_at).toLocaleDateString()}</td>
              <td>
                <div class="relative">
                  <button 
                    class="btn btn-sm btn-outline"
                    on:click={(e) => openActionDropdown(e, pool.id)}
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 0 110-2 1 0 010 2zm0 7a1 0 110-2 1 0 010 2zm0 7a1 0 110-2 1 0 010 2z" />
                    </svg>
                  </button>
                </div>
                {#if testResults.has(pool.id)}
                  {@const result = testResults.get(pool.id)}
                  <div class="mt-2 text-xs {result?.success ? 'text-success' : 'text-error'}">
                    {result?.success ? 'API可用' : result?.message}
                  </div>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <Pagination {page} {totalPages} onPageChange={goToPage} />
  {/if}
</div>

{#if showModal}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg mb-4">
        {editMode ? $_('tokenPools.modal.editTitle') : $_('tokenPools.modal.createTitle')}
      </h3>
      
      <div class="form-control mb-4">
        <label class="label" for="pool-name">
          <span class="label-text">{$_('tokenPools.modal.name')}</span>
        </label>
        <input 
          id="pool-name"
          type="text" 
          class="input input-bordered w-full"
          bind:value={formData.name}
          placeholder={$_('tokenPools.modal.namePlaceholder')}
          disabled={saving}
        />
      </div>
      
      <div class="form-control mb-4">
        <label class="label" for="pool-model">
          <span class="label-text">{$_('tokenPools.modal.modelType')}</span>
        </label>
        <ModelDropdown
          id="pool-model"
          selectedModel={formData.model_type}
          placeholder={$_('tokenPools.modal.selectModel')}
          searchPlaceholder={$_('tokenPools.modal.searchModels')}
          noModelsFound={$_('tokenPools.modal.noModelsFound')}
          onSelect={selectModel}
        />
      </div>

      <div class="form-control mb-4">
        <label class="label" for="pool-url">
          <span class="label-text">{$_('tokenPools.modal.baseUrl')}</span>
        </label>
        <input 
          id="pool-url"
          type="text" 
          class="input input-bordered w-full"
          bind:value={formData.base_url}
          placeholder={$_('tokenPools.modal.baseUrlPlaceholder')}
          disabled={saving}
        />
      </div>

      <div class="form-control mb-4">
        <label class="label" for="pool-key">
          <span class="label-text">{$_('tokenPools.modal.apiKey')}</span>
        </label>
        <input 
          id="pool-key"
          type="password" 
          class="input input-bordered w-full"
          bind:value={formData.api_key}
          placeholder={editMode ? $_('tokenPools.modal.apiKeyPlaceholderEdit') : $_('tokenPools.modal.apiKeyPlaceholder')}
          disabled={saving}
        />
        {#if editMode}
          <label class="label">
            <span class="label-text-alt text-base-content/50">{$_('tokenPools.modal.apiKeyHint')}</span>
          </label>
        {/if}
      </div>

      <div class="modal-action">
        <button class="btn btn-outline" on:click={closeModal}>{$_('tokenPools.modal.cancel')}</button>
        <button 
          class="btn btn-primary" 
          on:click={savePool}
          disabled={!formData.name.trim() || !formData.model_type.trim() || !formData.base_url.trim() || (!editMode && !formData.api_key.trim()) || saving}
        >
          {saving ? $_('tokenPools.modal.saving') : (editMode ? $_('tokenPools.modal.update') : $_('tokenPools.modal.create'))}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if actionDropdownOpen !== null}
  <div class="fixed inset-0 z-[9999]" on:click={closeActionDropdown}>
    <ul 
      class="absolute menu bg-base-100 rounded-box shadow-xl border border-base-300 p-2 w-40"
      style="top: {actionDropdownTop}px; left: {actionDropdownLeft}px;"
      on:click|stopPropagation
    >
      <li>
        <button on:click={handleEditAction}>
          {$_('tokenPools.actions.edit')}
        </button>
      </li>
      <li>
        <button 
          on:click={handleTestAction}
          disabled={testingId === actionDropdownOpen}
        >
          {testingId === actionDropdownOpen ? $_('tokenPools.test.testing') : $_('tokenPools.actions.test')}
        </button>
      </li>
      <li>
        <button on:click={handleChatAction}>
          {$_('tokenPools.actions.chat')}
        </button>
      </li>
      <li>
        <button on:click={handleToggleAction}>
          {getActionPool()?.is_active ? $_('tokenPools.actions.disable') : $_('tokenPools.actions.enable')}
        </button>
      </li>
      <li>
        <button class="text-error" on:click={handleDeleteAction}>
          {$_('tokenPools.actions.delete')}
        </button>
      </li>
    </ul>
  </div>
{/if}
