<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { _ } from 'svelte-i18n';
  import { t } from '$lib/i18n';

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
  let modelSearch = '';
  let modelDropdownOpen = false;
  let modelDropdownTop = 0;
  let modelDropdownLeft = 0;
  let modelDropdownWidth = 0;
  let saving = false;
  let testingId: number | null = null;
  let testResults: Map<number, { success: boolean; message: string; responseContent?: string }> = new Map();
  let actionDropdownOpen: number | null = null;
  let actionDropdownTop = 0;
  let actionDropdownLeft = 0;

  const commonModels = [
    { id: 'gpt-4o', name: 'GPT-4o', provider: 'OpenAI' },
    { id: 'gpt-4o-mini', name: 'GPT-4o Mini', provider: 'OpenAI' },
    { id: 'gpt-4-turbo', name: 'GPT-4 Turbo', provider: 'OpenAI' },
    { id: 'gpt-4', name: 'GPT-4', provider: 'OpenAI' },
    { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo', provider: 'OpenAI' },
    { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet', provider: 'Anthropic' },
    { id: 'claude-3-5-haiku-20241022', name: 'Claude 3.5 Haiku', provider: 'Anthropic' },
    { id: 'claude-3-opus-20240229', name: 'Claude 3 Opus', provider: 'Anthropic' },
    { id: 'gemini-1.5-pro', name: 'Gemini 1.5 Pro', provider: 'Google' },
    { id: 'gemini-1.5-flash', name: 'Gemini 1.5 Flash', provider: 'Google' },
    { id: 'gemini-2.0-flash', name: 'Gemini 2.0 Flash', provider: 'Google' },
    { id: 'llama-3.1-405b', name: 'Llama 3.1 405B', provider: 'Meta' },
    { id: 'llama-3.1-70b', name: 'Llama 3.1 70B', provider: 'Meta' },
    { id: 'llama-3.1-8b', name: 'Llama 3.1 8B', provider: 'Meta' },
    { id: 'mistral-large', name: 'Mistral Large', provider: 'Mistral' },
    { id: 'mistral-medium', name: 'Mistral Medium', provider: 'Mistral' },
    { id: 'codestral-latest', name: 'Codestral', provider: 'Mistral' },
    { id: 'deepseek-chat', name: 'DeepSeek Chat', provider: 'DeepSeek' },
    { id: 'deepseek-coder', name: 'DeepSeek Coder', provider: 'DeepSeek' },
  ];

  $: filteredModels = commonModels.filter(m => 
    m.name.toLowerCase().includes(modelSearch.toLowerCase()) ||
    m.id.toLowerCase().includes(modelSearch.toLowerCase()) ||
    m.provider.toLowerCase().includes(modelSearch.toLowerCase())
  );

  function selectModel(model: { id: string; name: string; provider: string }) {
    formData.model_type = model.id;
    modelDropdownOpen = false;
    modelSearch = '';
  }

  function closeModelDropdown() {
    modelDropdownOpen = false;
    modelSearch = '';
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

  function openModelDropdown(event: MouseEvent) {
    const target = event.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    modelDropdownTop = rect.bottom + 4;
    modelDropdownLeft = rect.left;
    modelDropdownWidth = rect.width;
    modelDropdownOpen = true;
    modelSearch = '';
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
    modelSearch = '';
    modelDropdownOpen = false;
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
                  <div class="mt-2 text-xs {testResults.get(pool.id)?.success ? 'text-success' : 'text-error'}">
                    {testResults.get(pool.id)?.message}
                    {#if testResults.get(pool.id)?.responseContent}
                      <div class="mt-2 p-2 bg-base-200 rounded text-base-content whitespace-pre-wrap max-h-60 overflow-y-auto">
                        {testResults.get(pool.id)?.responseContent}
                      </div>
                    {/if}
                  </div>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

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
        <button 
          type="button" 
          id="pool-model"
          class="input input-bordered w-full flex items-center cursor-pointer text-left"
          on:click={openModelDropdown}
        >
          {#if formData.model_type}
            <span>{commonModels.find(m => m.id === formData.model_type)?.name || formData.model_type}</span>
          {:else}
            <span class="text-base-content/50">{$_('tokenPools.modal.selectModel')}</span>
          {/if}
        </button>
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

{#if modelDropdownOpen}
  <div class="fixed inset-0 z-[9999]" on:click={closeModelDropdown}>
    <div 
      class="absolute bg-base-100 rounded-box shadow-xl border border-base-300 max-h-80 overflow-y-auto"
      style="width: {modelDropdownWidth}px; top: {modelDropdownTop}px; left: {modelDropdownLeft}px;"
      on:click|stopPropagation
    >
      <div class="p-2 border-b border-base-300">
        <input 
          type="text" 
          class="input input-sm input-bordered w-full"
          bind:value={modelSearch}
          placeholder={$_('tokenPools.modal.searchModels')}
        />
      </div>
      <ul class="menu p-2">
        {#each filteredModels as model}
          <li>
            <button type="button" on:click={() => selectModel(model)}>
              <div class="flex flex-col items-start">
                <span class="font-medium">{model.name}</span>
                <span class="text-xs opacity-60">{model.provider} · {model.id}</span>
              </div>
            </button>
          </li>
        {/each}
        {#if filteredModels.length === 0}
          <li class="menu-disabled"><span>{$_('tokenPools.modal.noModelsFound')}</span></li>
        {/if}
      </ul>
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
