import { error } from '@sveltejs/kit';
import createClient from 'openapi-fetch';
import { m } from '$paraglide/messages.js';
import { getLocale } from '$paraglide/runtime.js';
import type { components, paths } from './api.js';

export type Schemas = components['schemas'];
export type Card = Schemas['CardSummary'];
export type CardDetail = Schemas['CardDetail'];
export type CardMap = Map<string, Card>;
export type Client = ReturnType<typeof api>;

export const api = (fetch: typeof globalThis.fetch, baseUrl: string) =>
	createClient<paths>({ baseUrl, fetch });

export function ok<T>(res: { data?: T; response: Response }): T {
	if (res.response.ok && res.data !== undefined) return res.data;
	const notFound = res.response.status === 400 || res.response.status === 404;
	error(notFound ? 404 : 502, notFound ? 'Not Found' : 'API error');
}

export const percent = (value: number | null | undefined) =>
	value == null
		? '—'
		: new Intl.NumberFormat(getLocale(), { style: 'percent', maximumFractionDigits: 1 }).format(
				value
			);

export const entryName = (entry: { id: string; name?: string | null }) => entry.name ?? entry.id;

export const modeName = (mode: number) => (mode === 10 ? m.mode_ranked() : m.mode_n({ n: mode }));

export const formatDate = (iso: string) =>
	new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeZone: 'UTC' }).format(
		new Date(iso)
	);

export { fold, glossaryAnchor } from './description.js';
export { formatChange, winRateColor } from './change.js';
export { cn } from 'cn';

export type WithoutChild<T> = T extends { child?: unknown } ? Omit<T, 'child'> : T;
export type WithoutChildren<T> = T extends { children?: unknown } ? Omit<T, 'children'> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
