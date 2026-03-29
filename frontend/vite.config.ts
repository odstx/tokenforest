import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
	const env = loadEnv(mode, process.cwd(), '');
	const apiBaseUrl = env.VITE_API_BASE_URL || 'http://localhost:3000';
	
	return {
		plugins: [sveltekit()],
		server: {
			proxy: {
				'/api/': {
					target: apiBaseUrl,
					changeOrigin: true
				}
			}
		}
	};
});
