export interface Model {
  id: string;
  name: string;
  provider: string;
  base_url: string;
}

export const commonModels: Model[] = [
  { id: 'deepseek-chat', name: 'DeepSeek Chat', provider: 'DeepSeek', base_url: 'https://api.deepseek.com' },
  { id: 'deepseek-coder', name: 'DeepSeek Coder', provider: 'DeepSeek', base_url: 'https://api.deepseek.com' },
  { id: 'qwen3-max', name: 'Qwen3 Max', provider: 'Alibaba', base_url: 'https://dashscope.aliyuncs.com/compatible-mode/v1' },
  { id: 'qwen3.5-plus', name: 'Qwen3.5 Plus', provider: 'Alibaba', base_url: 'https://dashscope.aliyuncs.com/compatible-mode/v1' },
  { id: 'qwen3.5-flash', name: 'Qwen3.5 Flash', provider: 'Alibaba', base_url: 'https://dashscope.aliyuncs.com/compatible-mode/v1' },
];
