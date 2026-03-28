import type { Handle } from '@sveltejs/kit';

const API_BASE = 'http://localhost:3000';

export const handle: Handle = async ({ event, resolve }) => {
	if (event.url.pathname.startsWith('/api/')) {
		const targetUrl = `${API_BASE}${event.url.pathname}${event.url.search}`;
		
		const headers: Record<string, string> = {};
		event.request.headers.forEach((value, key) => {
			if (key.toLowerCase() !== 'host') {
				headers[key] = value;
			}
		});

		const response = await fetch(targetUrl, {
			method: event.request.method,
			headers,
			body: event.request.method !== 'GET' && event.request.method !== 'HEAD' 
				? await event.request.text() 
				: undefined
		});

		return new Response(response.body, {
			status: response.status,
			headers: {
				'content-type': response.headers.get('content-type') || 'application/json',
				'access-control-allow-origin': '*'
			}
		});
	}

	return resolve(event);
};
