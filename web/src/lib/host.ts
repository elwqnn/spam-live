import { resolve } from '$app/paths';
import type { Path } from '$app/types';
import { api } from '@spam-live/ui';
import { apiWrites } from '@spam-live/ui/writes.js';
import { localizeHref } from './paraglide/runtime.js';

export const href = (path: string) => resolve(localizeHref(path) as Path);

export const queryHref = (
	url: { pathname: string; search: string },
	changes: Record<string, string | null>
) => {
	const params = new URLSearchParams(url.search);
	for (const [key, value] of Object.entries(changes)) {
		if (value) params.set(key, value);
		else params.delete(key);
	}
	const search = params.toString().replaceAll('%2C', ',');
	return resolve((url.pathname + (search ? `?${search}` : '')) as Path);
};

export const loginHref = (url: { pathname: string; search: string }) =>
	`/api/auth/steam?redirect=${encodeURIComponent(url.pathname + url.search)}`;

export const client = () => api(fetch, location.origin);

export const apiBase = '';

export const art = (path: string) => `/art/${path}`;

export const writes = apiWrites(client);
