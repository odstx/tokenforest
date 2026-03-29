export interface Model {
  id: string;
  name: string;
  provider: string;
}

export const commonModels: Model[] = [
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
