<script lang="ts">
  import { commonModels, type Model } from './commonModels';
  
  export let selectedModel: string = '';
  export let placeholder: string = 'Select a model';
  export let searchPlaceholder: string = 'Search models...';
  export let noModelsFound: string = 'No models found';
  export let onSelect: (model: Model) => void = () => {};

  let isOpen = false;
  let search = '';
  let dropdownTop = 0;
  let dropdownLeft = 0;
  let dropdownWidth = 0;

  $: filteredModels = commonModels.filter(m => 
    m.name.toLowerCase().includes(search.toLowerCase()) ||
    m.id.toLowerCase().includes(search.toLowerCase()) ||
    m.provider.toLowerCase().includes(search.toLowerCase())
  );

  $: selectedModelName = selectedModel 
    ? commonModels.find(m => m.id === selectedModel)?.name || selectedModel 
    : null;

  function openDropdown(event: MouseEvent) {
    const target = event.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    dropdownTop = rect.bottom + 4;
    dropdownLeft = rect.left;
    dropdownWidth = rect.width;
    isOpen = true;
    search = '';
  }

  function closeDropdown() {
    isOpen = false;
    search = '';
  }

  function selectModel(model: Model) {
    onSelect(model);
    closeDropdown();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      isOpen = !isOpen;
    } else if (event.key === 'Escape') {
      closeDropdown();
    }
  }
</script>

<button 
  type="button" 
  class="input input-bordered w-full flex items-center cursor-pointer text-left"
  on:click={openDropdown}
  on:keydown={handleKeydown}
  aria-haspopup="listbox"
  aria-expanded={isOpen}
>
  {#if selectedModelName}
    <span>{selectedModelName}</span>
  {:else}
    <span class="text-base-content/50">{placeholder}</span>
  {/if}
</button>

{#if isOpen}
  <div class="fixed inset-0 z-[9999]" on:click={closeDropdown} on:keydown={() => {}} role="button" tabindex="-1">
    <div 
      class="absolute bg-base-100 rounded-box shadow-xl border border-base-300 max-h-80 overflow-y-auto"
      style="width: {dropdownWidth}px; top: {dropdownTop}px; left: {dropdownLeft}px;"
      on:click|stopPropagation
      role="listbox"
    >
      <div class="p-2 border-b border-base-300">
        <input 
          type="text" 
          class="input input-sm input-bordered w-full"
          bind:value={search}
          placeholder={searchPlaceholder}
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
          <li class="menu-disabled"><span>{noModelsFound}</span></li>
        {/if}
      </ul>
    </div>
  </div>
{/if}
