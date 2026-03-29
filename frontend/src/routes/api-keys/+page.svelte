<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { _ } from 'svelte-i18n';
  import { t } from '$lib/i18n';
  import { commonModels, type Model } from '$lib/components/commonModels';
  import ModelDropdown from '$lib/components/ModelDropdown.svelte';
  import Pagination from '$lib/components/Pagination.svelte';

  interface ApiKey {
    id: number;
    name: string;
    model: string | null;
    prefix: string;
    is_active: boolean;
    last_used_at: string | null;
    created_at: string;
    allowed_cidrs?: string[];
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
  let newKeyCidrs: string[] = [];
  let cidrInput = '';
  let actionDropdownOpen: number | null = null;
  let actionDropdownTop = 0;
  let actionDropdownLeft = 0;
  let creatingKey = false;
  let createdKey: string | null = null;
  let editMode = false;
  let editingId: number | null = null;
  let savingKey = false;

  function selectModel(model: Model) {
    newKeyModel = model.id;
  }

  function openActionDropdown(event: MouseEvent, keyId: number) {
    const target = event.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    actionDropdownTop = rect.bottom + 4;
    actionDropdownLeft = rect.right - 160;
    actionDropdownOpen = keyId;
  }

  function closeActionDropdown() {
    actionDropdownOpen = null;
  }

  function handleToggleAction() {
    const keyId = actionDropdownOpen;
    if (keyId !== null) {
      closeActionDropdown();
      toggleKey(keyId);
    }
  }

  function handleDeleteAction() {
    const keyId = actionDropdownOpen;
    if (keyId !== null) {
      closeActionDropdown();
      deleteKey(keyId);
    }
  }

  function getActionKey() {
    return apiKeys.find(k => k.id === actionDropdownOpen);
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
          model: newKeyModel || null,
          allowed_cidrs: newKeyCidrs.length > 0 ? newKeyCidrs : null
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

  function addCidr() {
    const cidr = cidrInput.trim();
    if (cidr && !newKeyCidrs.includes(cidr)) {
      newKeyCidrs = [...newKeyCidrs, cidr];
      cidrInput = '';
    }
  }

  function removeCidr(cidr: string) {
    newKeyCidrs = newKeyCidrs.filter(c => c !== cidr);
  }

  function handleEditAction() {
    const key = getActionKey();
    if (key) {
      editMode = true;
      editingId = key.id;
      newKeyName = key.name;
      newKeyModel = key.model || '';
      newKeyCidrs = key.allowed_cidrs || [];
      closeActionDropdown();
      showModal = true;
    }
  }

  async function saveKey() {
    if (!browser || !newKeyName.trim()) return;
    
    const token = localStorage.getItem('token');
    if (!token) return;

    savingKey = true;
    error = null;

    try {
      const response = await fetch(`/api/api-keys/${editingId}`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ 
          name: newKeyName.trim(),
          model: newKeyModel || null,
          allowed_cidrs: newKeyCidrs.length > 0 ? newKeyCidrs : null
        })
      });

      if (!response.ok) {
        throw new Error('Failed to update API key');
      }

      closeModal();
      await fetchApiKeys();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to update API key';
    } finally {
      savingKey = false;
    }
  }

  function closeModal() {
    showModal = false;
    createdKey = null;
    editMode = false;
    editingId = null;
    newKeyName = '';
    newKeyModel = '';
    newKeyCidrs = [];
    cidrInput = '';
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
            <th>{$_('apiKeys.table.cidrRestrictions')}</th>
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
              <td class="whitespace-nowrap">{key.name}</td>
              <td class="whitespace-nowrap">
                {#if key.model}
                  <span class="badge badge-outline badge-sm">{key.model}</span>
                {:else}
                  <span class="text-base-content/50">{$_('apiKeys.model.any')}</span>
                {/if}
              </td>
              <td class="whitespace-nowrap">
                {#if key.allowed_cidrs && key.allowed_cidrs.length > 0}
                  {#each key.allowed_cidrs as cidr}
                    <span class="badge badge-outline badge-sm mr-1">{cidr}</span>
                  {/each}
                {:else}
                  <span class="text-base-content/50">{$_('apiKeys.cidr.any')}</span>
                {/if}
              </td>
              <td class="whitespace-nowrap"><code class="bg-base-300 px-2 py-1 rounded">{key.prefix}...</code></td>
              <td class="whitespace-nowrap">
                <span class="badge {key.is_active ? 'badge-success' : 'badge-error'}">
                  {key.is_active ? $_('apiKeys.status.active') : $_('apiKeys.status.inactive')}
                </span>
              </td>
              <td class="whitespace-nowrap">{key.last_used_at || $_('apiKeys.lastUsed.never')}</td>
              <td class="whitespace-nowrap">{new Date(key.created_at).toLocaleDateString()}</td>
              <td class="whitespace-nowrap">
                <div class="relative">
                  <button 
                    class="btn btn-sm btn-outline"
                    on:click={(e) => openActionDropdown(e, key.id)}
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 0 110-2 1 0 110 2zm0 7a1 0 110-2 1 0 110 2zm0 7a1 0 110-2 1 0 110 2z" />
                    </svg>
                  </button>
                </div>
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
      <h3 class="font-bold text-lg mb-4">{editMode ? $_('apiKeys.modal.editTitle') : $_('apiKeys.modal.title')}</h3>
      
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
          <ModelDropdown
            selectedModel={newKeyModel}
            placeholder={$_('apiKeys.modal.selectModel')}
            searchPlaceholder={$_('apiKeys.modal.searchModels')}
            noModelsFound={$_('apiKeys.modal.noModelsFound')}
            onSelect={selectModel}
          />
        </div>
        
        <div class="form-control mb-4">
          <label class="label">
            <span class="label-text">{$_('apiKeys.cidr.title')}</span>
          </label>
          <div class="flex gap-2 mb-2">
            <input 
              type="text" 
              class="input input-bordered flex-1"
              bind:value={cidrInput}
              placeholder={$_('apiKeys.cidr.placeholder')}
              on:keydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); addCidr(); } }}
            />
            <button type="button" class="btn btn-outline" on:click={addCidr}>
              {$_('apiKeys.cidr.add')}
            </button>
          </div>
          {#if newKeyCidrs.length > 0}
            <div class="flex flex-wrap gap-2">
              {#each newKeyCidrs as cidr}
                <span class="badge badge-outline gap-1">
                  {cidr}
                  <button type="button" class="btn btn-ghost btn-xs p-0 h-auto min-h-0" on:click={() => removeCidr(cidr)}>
                    ×
                  </button>
                </span>
              {/each}
            </div>
          {/if}
          <label class="label">
            <span class="label-text-alt text-base-content/60">{$_('apiKeys.cidr.hint')}</span>
          </label>
        </div>
      {/if}

      <div class="modal-action">
        {#if createdKey}
          <button class="btn" on:click={closeModal}>{$_('apiKeys.modal.done')}</button>
        {:else if editMode}
          <button class="btn btn-outline" on:click={closeModal}>{$_('apiKeys.modal.cancel')}</button>
          <button 
            class="btn btn-primary" 
            on:click={saveKey}
            disabled={!newKeyName.trim() || savingKey}
          >
            {savingKey ? $_('apiKeys.modal.saving') : $_('apiKeys.modal.save')}
          </button>
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

{#if actionDropdownOpen !== null}
  <div class="fixed inset-0 z-[9999]" on:click={closeActionDropdown}>
    <ul 
      class="absolute menu bg-base-100 rounded-box shadow-xl border border-base-300 p-2 w-40"
      style="top: {actionDropdownTop}px; left: {actionDropdownLeft}px;"
      on:click|stopPropagation
    >
      <li>
        <button on:click={handleEditAction}>
          {$_('apiKeys.actions.edit')}
        </button>
      </li>
      <li>
        <button on:click={handleToggleAction}>
          {getActionKey()?.is_active ? $_('apiKeys.actions.disable') : $_('apiKeys.actions.enable')}
        </button>
      </li>
      <li>
        <button class="text-error" on:click={handleDeleteAction}>
          {$_('apiKeys.actions.delete')}
        </button>
      </li>
    </ul>
  </div>
{/if}
