<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { page } from '$app/stores';
  import { _ } from 'svelte-i18n';

  interface Message {
    role: 'user' | 'assistant';
    content: string;
  }

  let inputText = '';
  let sending = false;
  let error: string | null = null;
  let loading = true;
  let resourceName = '';
  let resourceType = '';
  let resourceId: number | null = null;
  let selectedModel = '';
  let messages: Message[] = [];
  let messagesContainer: HTMLDivElement;
  let apiKey = '';

  async function fetchResourceInfo() {
    if (!browser) return;
    
    const token = localStorage.getItem('token');
    if (!token) {
      goto('/login');
      return;
    }

    const type = $page.url.searchParams.get('type');
    const id = $page.url.searchParams.get('id');

    if (!type || !id || (type !== 'token-pool' && type !== 'api-key')) {
      goto('/token-pools');
      return;
    }

    resourceType = type;
    resourceId = parseInt(id);

    if (type === 'token-pool') {
      try {
        const response = await fetch(`/api/token-pools/${id}`, {
          headers: { 'Authorization': `Bearer ${token}` }
        });
        if (response.ok) {
          const data = await response.json();
          resourceName = data.name;
          if (data.model_type) {
            selectedModel = data.model_type;
          }
        } else {
          error = 'Token pool not found';
        }
      } catch {
        error = 'Failed to load token pool';
      }
    } else if (type === 'api-key') {
      try {
        const [infoRes, keyRes] = await Promise.all([
          fetch(`/api/api-keys/${id}`, {
            headers: { 'Authorization': `Bearer ${token}` }
          }),
          fetch(`/api/api-keys/${id}/key`, {
            headers: { 'Authorization': `Bearer ${token}` }
          })
        ]);
        if (infoRes.ok && keyRes.ok) {
          const data = await infoRes.json();
          const keyData = await keyRes.json();
          resourceName = data.name;
          if (data.model) {
            selectedModel = data.model;
          }
          apiKey = keyData.key;
        } else if (!keyRes.ok) {
          const keyData = await keyRes.json();
          if (keyData.error?.includes('no stored key value')) {
            error = 'This API key was created before encryption was enabled. Please create a new API key to use chat.';
          } else {
            error = 'Failed to retrieve API key value';
          }
        } else {
          error = 'API key not found';
        }
      } catch {
        error = 'Failed to load API key';
      }
    }

    loading = false;
  }

  async function sendMessage() {
    if (!browser || !inputText.trim() || sending || resourceId === null) return;

    const userMessage = inputText.trim();
    inputText = '';
    sending = true;
    error = null;

    messages = [...messages, { role: 'user', content: userMessage }];
    scrollToBottom();

    try {
      if (resourceType === 'token-pool') {
        await sendTokenPoolMessage(userMessage);
      } else if (resourceType === 'api-key') {
        await sendApiKeyMessage(userMessage);
      }
    } catch (err) {
      messages = messages.slice(0, -1);
      error = err instanceof Error ? err.message : 'Failed to send message';
    } finally {
      sending = false;
    }
  }

  async function sendTokenPoolMessage(_userMessage: string) {
    const token = localStorage.getItem('token');
    if (!token) return;

    const response = await fetch(`/api/token-pools/${resourceId}/chat`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model: selectedModel,
        messages: messages.map(m => ({ role: m.role, content: m.content })),
        max_tokens: 1000
      })
    });

    const data = await response.json();
    
    if (data.success && data.content) {
      messages = [...messages, { role: 'assistant', content: data.content }];
      scrollToBottom();
    } else {
      messages = messages.slice(0, -1);
      error = data.error || 'Failed to get response';
    }
  }

  async function sendApiKeyMessage(_userMessage: string) {
    const response = await fetch('http://localhost:8000/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${apiKey}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model: selectedModel,
        messages: messages.map(m => ({ role: m.role, content: m.content })),
        max_tokens: 1000,
        stream: true
      })
    });

    if (!response.ok) {
      const data = await response.json();
      messages = messages.slice(0, -1);
      error = data.error?.message || data.error || 'Failed to get response';
      return;
    }

    const reader = response.body?.getReader();
    if (!reader) {
      messages = messages.slice(0, -1);
      error = 'No response body';
      return;
    }

    const decoder = new TextDecoder();
    let assistantContent = '';
    messages = [...messages, { role: 'assistant', content: '' }];

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const chunk = decoder.decode(value, { stream: true });
      const lines = chunk.split('\n');

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6);
          if (data === '[DONE]') continue;
          
          try {
            const parsed = JSON.parse(data);
            const delta = parsed.choices?.[0]?.delta?.content;
            if (delta) {
              assistantContent += delta;
              messages = [...messages.slice(0, -1), { role: 'assistant', content: assistantContent }];
              scrollToBottom();
            }
          } catch {
            // Ignore parse errors for incomplete chunks
          }
        }
      }
    }
  }

  function scrollToBottom() {
    setTimeout(() => {
      if (messagesContainer) {
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
      }
    }, 0);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  function clearChat() {
    messages = [];
    error = null;
  }

  onMount(fetchResourceInfo);
</script>

<div class="flex flex-col h-screen">
  <div class="flex items-center justify-between p-4 border-b border-base-300 shrink-0">
    <div class="flex items-center gap-4">
      <button class="btn btn-sm btn-ghost" on:click={() => goto(resourceType === 'api-key' ? '/api-keys' : '/token-pools')}>
        ← {$_('chat.back')}
      </button>
      <h1 class="text-xl font-bold">{$_('chat.title')}</h1>
    </div>
    <div class="flex items-center gap-2">
      <span class="text-sm text-base-content/70">
        {resourceType === 'token-pool' ? $_('chat.usingPool', { values: { name: resourceName } }) : $_('chat.usingKey', { values: { name: resourceName } })}
      </span>
      {#if messages.length > 0}
        <button class="btn btn-sm btn-ghost" on:click={clearChat}>
          {$_('chat.clear')}
        </button>
      {/if}
    </div>
  </div>

  {#if loading}
    <div class="flex-1 flex items-center justify-center">
      <span class="loading loading-spinner loading-lg"></span>
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto p-4 pb-32 min-h-0" bind:this={messagesContainer}>
      {#if messages.length === 0}
        <div class="flex items-center justify-center h-full">
          <p class="text-base-content/50">{$_('chat.noMessages')}</p>
        </div>
      {:else}
        <div class="max-w-3xl mx-auto space-y-4">
          {#each messages as message}
            {#if message.role === 'user'}
              <div class="flex justify-end">
                <div class="max-w-[80%] rounded-lg p-3 bg-green-200 text-green-900">
                  <pre class="whitespace-pre-wrap font-sans text-sm">{message.content}</pre>
                </div>
              </div>
            {:else}
              <div class="flex justify-start">
                <div class="max-w-[80%] rounded-lg p-3 bg-base-200">
                  <pre class="whitespace-pre-wrap font-sans text-sm">{message.content}</pre>
                </div>
              </div>
            {/if}
          {/each}
          {#if sending}
            <div class="flex justify-start">
              <div class="max-w-[80%] rounded-lg p-3 bg-base-200">
                <span class="loading loading-dots loading-sm"></span>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    {#if error}
      <div class="px-4 pb-2 fixed bottom-[76px] left-0 right-0">
        <div class="max-w-3xl mx-auto">
          <div class="alert alert-error">
            <span>{error}</span>
          </div>
        </div>
      </div>
    {/if}

    <div class="fixed bottom-0 left-0 right-0 p-4 border-t border-base-300 bg-base-100">
      <div class="max-w-3xl mx-auto">
        <div class="flex gap-2">
          <textarea
            class="textarea textarea-bordered flex-1 resize-none"
            rows="2"
            bind:value={inputText}
            on:keydown={handleKeydown}
            placeholder={$_('chat.placeholder')}
            disabled={sending}
          ></textarea>
          <button
            class="btn btn-primary"
            on:click={sendMessage}
            disabled={!inputText.trim() || sending}
          >
            {sending ? $_('chat.sending') : $_('chat.send')}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
