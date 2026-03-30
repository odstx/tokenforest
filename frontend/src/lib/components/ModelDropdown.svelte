<script lang="ts">
  import { commonModels, type Model } from './commonModels';
  
  export let id: string = '';
  export let selectedModel: string = '';
  export let placeholder: string = 'Select a model';
  export let searchPlaceholder: string = 'Search models...';
  export let noModelsFound: string = 'No models found';
  export let onSelect: (model: Model) => void = () => {};
  export let onManualInput: (value: string) => void = () => {};

  let isOpen = false;
  let search = '';
  let dropdownTop = 0;
  let dropdownLeft = 0;
  let dropdownWidth = 0;
  let inputRef: HTMLInputElement | null = null;

  $: filteredModels = commonModels.filter(m => 
    m.name.toLowerCase().includes(search.toLowerCase()) ||
    m.id.toLowerCase().includes(search.toLowerCase()) ||
    m.provider.toLowerCase().includes(search.toLowerCase())
  );

  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        if (node.parentNode) {
          node.parentNode.removeChild(node);
        }
      }
    };
  }

  function openDropdown() {
    if (inputRef) {
      const rect = inputRef.getBoundingClientRect();
      dropdownTop = rect.bottom + 4;
      dropdownLeft = rect.left;
      dropdownWidth = rect.width;
    }
    isOpen = true;
  }

  function closeDropdown() {
    isOpen = false;
    search = '';
  }

  function selectModel(model: Model) {
    onSelect(model);
    search = '';
    closeDropdown();
  }

  function handleFocus() {
    openDropdown();
  }

  function handleInput(event: Event) {
    const value = (event.target as HTMLInputElement).value;
    if (!isOpen) {
      openDropdown();
    }
    onManualInput(value);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeDropdown();
    }
  }
</script>

<div class="relative">
  <input 
    type="text"
    class="input input-bordered w-full"
    {id}
    bind:this={inputRef}
    bind:value={selectedModel}
    {placeholder}
    on:focus={handleFocus}
    on:input={handleInput}
    on:keydown={handleKeydown}
    autocomplete="off"
  />
</div>

{#if isOpen}
  <div class="fixed inset-0 z-[9999]" on:click={closeDropdown} role="button" tabindex="-1" use:portal></div>
  <div 
    class="fixed bg-base-100 rounded-box shadow-xl border border-base-300 max-h-80 overflow-y-auto z-[10000]"
    style="width: {dropdownWidth}px; top: {dropdownTop}px; left: {dropdownLeft}px;"
    on:click|stopPropagation
    role="listbox"
    use:portal
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
{/if}
