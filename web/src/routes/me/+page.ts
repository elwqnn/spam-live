import { redirect } from '@sveltejs/kit';
import { api, loginHref, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

const PAGE_SIZE = 24;

export const load: PageLoad = async ({ fetch, url, parent, depends }) => {
	depends('app:decks');
	const { me } = await parent();
	if (!me) redirect(303, loginHref(url));
	const client = api(fetch, url.origin);
	const before = url.searchParams.get('before') || undefined;
	const [cards, matches, decks, devices] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang: getLocale() } } }),
		client.GET('/api/me/matches', { params: { query: { before, limit: PAGE_SIZE + 1 } } }),
		client.GET('/api/decks', { params: { query: { author: 'me', sort: 'recent', limit: 6 } } }),
		client.GET('/api/me/devices')
	]);
	const list = ok(matches);
	return {
		me,
		cards: ok(cards),
		matches: list.slice(0, PAGE_SIZE),
		next: list.length > PAGE_SIZE ? list[PAGE_SIZE - 1].startUtc : null,
		first: !before,
		decks: ok(decks),
		devices: ok(devices)
	};
};
