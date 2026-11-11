import { api } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, untrack, data }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const [cards, top, recent, stats, players] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang: getLocale() } } }),
		client.GET('/api/stats/decks', { params: { query: { sort: 'winrate', limit: 6 } } }),
		client.GET('/api/decks', { params: { query: { sort: 'recent', limit: 12 } } }),
		client.GET('/api/stats/cards'),
		client.GET('/api/steam/players')
	]);
	return {
		...data,
		cards: cards.data ?? [],
		top: top.data ?? [],
		recent: recent.data ?? [],
		stats: stats.data ?? [],
		players: players.data?.current ?? null
	};
};
