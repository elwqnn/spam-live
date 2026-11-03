import { api } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const [cards, top, recent, versions] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang: getLocale() } } }),
		client.GET('/api/stats/decks', { params: { query: { sort: 'winrate', limit: 5 } } }),
		client.GET('/api/decks', { params: { query: { sort: 'recent', limit: 5 } } }),
		client.GET('/api/catalog/versions')
	]);
	return {
		cards: cards.data ?? [],
		top: (top.data ?? []).filter((d) => d.winRate != null),
		recent: recent.data ?? [],
		version: versions.data?.[0]?.version ?? null
	};
};
