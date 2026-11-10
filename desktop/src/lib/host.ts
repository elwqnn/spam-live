import { invoke, isTauri } from '@tauri-apps/api/core';
import { resolve } from '$app/paths';
import type { Path, ResolvedPathname } from '$app/types';
import { api } from '@spam-live/ui';
import { apiWrites, type Writes } from '@spam-live/ui/writes.js';
import { getLocale } from './paraglide/runtime.js';
import { SITE, account, appRoute, pathname, siteUrl } from './app.js';

type Answer = { status: number; contentType: string | null; body: string };

const EMPTY = [204, 205, 304];

export const href = (path: string) =>
	appRoute(pathname(path))
		? resolve(path as Path)
		: (siteUrl(path, getLocale()) as ResolvedPathname);

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

export const loginHref = () => `${resolve('/settings')}?link`;

async function viaApp(request: Request) {
	const url = new URL(request.url);
	const headers: Record<string, string> = {
		'x-method': request.method,
		'x-path': url.pathname + url.search
	};
	const type = request.headers.get('content-type');
	if (type) headers['x-type'] = type;
	const body = new Uint8Array(await request.arrayBuffer());
	const res = await invoke<Answer>('api', body, { headers }).catch((e) => {
		throw new TypeError(String(e));
	});
	return new Response(EMPTY.includes(res.status) ? null : res.body, {
		status: res.status,
		headers: res.contentType ? { 'content-type': res.contentType } : undefined
	});
}

const route: typeof fetch = (input, init) => {
	const request = new Request(input, init);
	return isTauri() && account(request.method, new URL(request.url).pathname)
		? viaApp(request)
		: fetch(request);
};

export const client = () => api(route, SITE);

export const apiBase = SITE;

export const art = (path: string) => `${SITE}/art/${path}`;

export const writes: Writes = {
	...apiWrites(client),
	logout: () => invoke('unlink').then(() => undefined)
};
