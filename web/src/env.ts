import { defineEnvVars } from '@sveltejs/kit/env';

export const variables = defineEnvVars({
	API_URL: {
		description: 'Base URL of the API for server-side requests',
		schema: (value) => (value || 'http://localhost:8080').replace(/\/$/, '')
	}
});
