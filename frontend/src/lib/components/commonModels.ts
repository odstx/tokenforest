export interface Model {
  id: string;
  name: string;
  provider: string;
}

export const commonModels: Model[] = [
  { id: 'deepseek-chat', name: 'DeepSeek Chat', provider: 'DeepSeek' },
  { id: 'deepseek-coder', name: 'DeepSeek Coder', provider: 'DeepSeek' },
  { id: 'qwen3-max', name: 'Qwen3 Max', provider: 'Alibaba' },
  { id: 'qwen3.5-plus', name: 'Qwen3.5 Plus', provider: 'Alibaba' },
  { id: 'qwen3.5-flash', name: 'Qwen3.5 Flash', provider: 'Alibaba' },
];
