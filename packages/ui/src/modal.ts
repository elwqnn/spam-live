import { page } from '$app/state';
import { locales } from '$paraglide/runtime.js';

export type Modal = { kind: 'card' | 'deck'; href: string; data: Record<string, unknown> };

const path = new RegExp(`^(?:/(?:${locales.join('|')}))?/(cards|decks)/([^/]+)/?$`);

export function modalKind(pathname: string): Modal['kind'] | null {
	const [, type, id] = path.exec(pathname) ?? [];
	if (type === 'cards') return 'card';
	if (type === 'decks' && /^\d+$/.test(id)) return 'deck';
	return null;
}

export const pageUrl = () => page.shallow?.url ?? page.url;
