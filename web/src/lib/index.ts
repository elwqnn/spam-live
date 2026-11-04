import { error, fail } from '@sveltejs/kit';
import createClient from 'openapi-fetch';
import type { SubmitFunction } from '$app/forms';
import { resolve } from '$app/paths';
import type { Path } from '$app/types';
import { m } from '#lib/paraglide/messages.js';
import { getLocale, localizeHref } from '#lib/paraglide/runtime.js';
import type { components, paths } from './api.js';

export type Schemas = components['schemas'];
export type Card = Schemas['CardSummary'];
export type CardDetail = Schemas['CardDetail'];
export type CardMap = Map<string, Card>;

export const api = (fetch: typeof globalThis.fetch, origin: string) =>
	createClient<paths>({ baseUrl: origin, fetch });

export function ok<T>(res: { data?: T; response: Response }): T {
	if (res.response.ok && res.data !== undefined) return res.data;
	const notFound = res.response.status === 400 || res.response.status === 404;
	error(notFound ? 404 : 502, notFound ? 'Not Found' : 'API error');
}

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

export const formAction = (url: { search: string }, name: string) =>
	`?/${name}${url.search.replace('?', '&')}`;

export const percent = (value: number | null | undefined) =>
	value == null
		? '—'
		: new Intl.NumberFormat(getLocale(), { style: 'percent', maximumFractionDigits: 1 }).format(
				value
			);

export const entryName = (entry: { id: string; name?: string | null }) => entry.name ?? entry.id;

export const loginHref = (url: { pathname: string; search: string }) =>
	`/api/auth/steam?redirect=${encodeURIComponent(url.pathname + url.search)}`;

export const modeName = (mode: number) => (mode === 10 ? m.mode_ranked() : m.mode_n({ n: mode }));

export const formatDate = (iso: string) =>
	new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeZone: 'UTC' }).format(
		new Date(iso)
	);

export function result(key: string, res: { error?: unknown; response: Response }) {
	if (res.response.ok) return { key, error: null };
	const body = res.error as { error?: string } | undefined;
	return fail(res.response.status, { key, error: body?.error ?? res.response.statusText });
}

export const closeAfter: SubmitFunction =
	({ submitter }) =>
	async ({ result, update, formElement }) => {
		(submitter?.closest('dialog') ?? formElement.closest('dialog'))?.close();
		if (result.type === 'success') formElement.closest('details')?.removeAttribute('open');
		await update();
	};

export { fold } from './description.js';
export { formatChange } from './change.js';
