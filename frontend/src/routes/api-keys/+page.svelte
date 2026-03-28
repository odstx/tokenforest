<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { _ } from 'svelte-i18n';
  import { t } from '$lib/i18n';

  interface ApiKey {
    id: number;
    name: string;
    model: string | null;
    prefix: string;
    is_active: boolean;
    last_used_at: string | null;
    created_at: string;
  }

  interface PaginatedResponse {
    items: ApiKey[];
    total: number;
    page: number;
    page_size: number;
    total_pages: number;
  }

  let apiKeys: ApiKey[] = [];
  let loading = true;
  let error: string | null = null;
  let showModal = false;
  let newKeyName = '';
  let newKeyModel = '';
  let modelSearch = '';
  let modelDropdownOpen = false;
  let modelDropdownTop = 0;
  let modelDropdownLeft = 0;
  let modelDropdownWidth = 0;
  let creatingKey = false;
  let createdKey: string | null = null;

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
    newKeyModel = model.id;
    modelDropdownOpen = false;
    modelSearch = '';
  }

  function closeModelDropdown() {
    modelDropdownOpen = false;
    modelSearch = '';
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

  async function fetchApiKeys() {
    if (!browser) return;
    
    const token = localStorage.getItem('token');
    if (!token) {
      goto('/login');
      return;
    }

    try {
      const response = await fetch(`/api/api-keys?page=${page}&page_size=${pageSize}`, {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });

      if (!response.ok) {
        if (response.status === 401) {
          goto('/login');
          return;
        }
        throw new Error('Failed to fetch API keys');
      }

      const data: PaginatedResponse = await response.json();
      console.log('API response:', data);
      apiKeys = data.items || [];
      total = data.total || 0;
      totalPages = data.total_pages || 0;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load API keys';
    } finally {
      loading = false;
    }
  }

  async function goToPage(p: number) {
    if (p < 1 || p > totalPages) return;
    page = p;
    loading = true;
    await fetchApiKeys();
  }

  async function createKey() {
    if (!browser || !newKeyName.trim()) return;
    
    const token = localStorage.getItem('token');
    if (!token) return;

    creatingKey = true;
    error = null;

    try {
      const response = await fetch('/api/api-keys', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ 
          name: newKeyName.trim(),
          model: newKeyModel || null
        })
      });

      if (!response.ok) {
        throw new Error('Failed to create API key');
      }

      const data = await response.json();
      createdKey = data.key;
      newKeyName = '';
      await fetchApiKeys();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to create API key';
    } finally {
      creatingKey = false;
    }
  }

  async function deleteKey(id: number) {
    if (!browser) return;
    if (!confirm(t('apiKeys.confirmDelete'))) return;
    
    const token = localStorage.getItem('token');
    if (!token) return;

    try {
      const response = await fetch(`/api/api-keys/${id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });

      if (!response.ok) {
        throw new Error('Failed to delete API key');
      }

      if (apiKeys.length === 1 && page > 1) {
        page--;
      }
      await fetchApiKeys();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to delete API key';
    }
  }

  async function toggleKey(id: number) {
    if (!browser) return;
    
    const token = localStorage.getItem('token');
    if (!token) return;

    try {
      const response = await fetch(`/api/api-keys/${id}/toggle`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });

      if (!response.ok) {
        throw new Error('Failed to toggle API key');
      }

      await fetchApiKeys();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to toggle API key';
    }
  }

  function closeModal() {
    showModal = false;
    createdKey = null;
    newKeyName = '';
    newKeyModel = '';
    modelSearch = '';
    modelDropdownOpen = false;
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }

  onMount(fetchApiKeys);
</script>

<div class="p-4 max-w-4xl mx-auto">
  <div class="flex justify-between items-center mb-6">
    <h1 class="text-2xl font-bold">{$_('apiKeys.title')}</h1>
    <button class="btn btn-primary" on:click={() => showModal = true}>
      {$_('apiKeys.createButton')}
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
  {:else if apiKeys.length === 0}
    <div class="text-center py-8 text-base-content/70">
      <p>{$_('apiKeys.noKeys')}</p>
    </div>
  {:else}
    <div class="overflow-x-auto">
      <table class="table table-zebra w-full">
        <thead>
          <tr>
            <th>{$_('apiKeys.table.name')}</th>
            <th>{$_('apiKeys.table.model')}</th>
            <th>{$_('apiKeys.table.keyPrefix')}</th>
            <th>{$_('apiKeys.table.status')}</th>
            <th>{$_('apiKeys.table.lastUsed')}</th>
            <th>{$_('apiKeys.table.created')}</th>
            <th>{$_('apiKeys.table.actions')}</th>
          </tr>
        </thead>
        <tbody>
          {#each apiKeys as key}
            <tr>
              <td>{key.name}</td>
              <td>
                {#if key.model}
                  <span class="badge badge-outline badge-sm">{key.model}</span>
                {:else}
                  <span class="text-base-content/50">{$_('apiKeys.model.any')}</span>
                {/if}
              </td>
              <td><code class="bg-base-300 px-2 py-1 rounded">{key.prefix}...</code></td>
              <td>
                <span class="badge {key.is_active ? 'badge-success' : 'badge-error'}">
                  {key.is_active ? $_('apiKeys.status.active') : $_('apiKeys.status.inactive')}
                </span>
              </td>
              <td>{key.last_used_at || $_('apiKeys.lastUsed.never')}</td>
              <td>{new Date(key.created_at).toLocaleDateString()}</td>
              <td>
                <div class="flex gap-2">
                  <button 
                    class="btn btn-sm btn-outline"
                    on:click={() => toggleKey(key.id)}
                  >
                    {key.is_active ? $_('apiKeys.actions.disable') : $_('apiKeys.actions.enable')}
                  </button>
                  <button 
                    class="btn btn-sm btn-error btn-outline"
                    on:click={() => deleteKey(key.id)}
                  >
                    {$_('apiKeys.actions.delete')}
                  </button>
                </div>
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
      <h3 class="font-bold text-lg mb-4">{$_('apiKeys.modal.title')}</h3>
      
      {#if createdKey}
        <div class="alert alert-success mb-4">
          <div>
            <p class="font-bold">{$_('apiKeys.modal.created')}</p>
            <p class="text-sm">{$_('apiKeys.modal.copyWarning')}</p>
          </div>
        </div>
        <div class="flex gap-2 mb-4">
          <code class="bg-base-300 px-3 py-2 rounded flex-1 break-all">{createdKey}</code>
          <button 
            class="btn btn-sm btn-outline"
            on:click={() => { if (createdKey) copyToClipboard(createdKey); }}
          >
            {$_('apiKeys.modal.copy')}
          </button>
        </div>
      {:else}
        <div class="form-control mb-4">
          <label class="label" for="key-name">
            <span class="label-text">{$_('apiKeys.modal.keyName')}</span>
          </label>
          <input 
            id="key-name"
            type="text" 
            class="input input-bordered w-full"
            bind:value={newKeyName}
            placeholder={$_('apiKeys.modal.keyNamePlaceholder')}
            disabled={creatingKey}
          />
        </div>
        
        <div class="form-control mb-4">
          <label class="label" for="key-model">
            <span class="label-text">{$_('apiKeys.modal.modelOptional')}</span>
          </label>
          <button 
            type="button" 
            id="key-model"
            class="input input-bordered w-full flex items-center cursor-pointer text-left"
            on:click={openModelDropdown}
          >
            {#if newKeyModel}
              <span>{commonModels.find(m => m.id === newKeyModel)?.name || newKeyModel}</span>
            {:else}
              <span class="text-base-content/50">{$_('apiKeys.modal.selectModel')}</span>
            {/if}
          </button>
        </div>
      {/if}

      <div class="modal-action">
        {#if createdKey}
          <button class="btn" on:click={closeModal}>{$_('apiKeys.modal.done')}</button>
        {:else}
          <button class="btn btn-outline" on:click={closeModal}>{$_('apiKeys.modal.cancel')}</button>
          <button 
            class="btn btn-primary" 
            on:click={createKey}
            disabled={!newKeyName.trim() || creatingKey}
          >
            {creatingKey ? $_('apiKeys.modal.creating') : $_('apiKeys.modal.create')}
          </button>
        {/if}
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
          placeholder={$_('apiKeys.modal.searchModels')}
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
          <li class="menu-disabled"><span>{$_('apiKeys.modal.noModelsFound')}</span></li>
        {/if}
      </ul>
    </div>
  </div>
{/if}
