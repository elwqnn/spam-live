import { api, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const version = url.searchParams.get('version') ?? undefined;
	const sort = url.searchParams.get('sort') === 'games' ? 'games' : 'winrate';
	const [cards, versions, decks, cardStats] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang: getLocale() } } }),
		client.GET('/api/catalog/versions'),
		client.GET('/api/stats/decks', { params: { query: { version, sort } } }),
		client.GET('/api/stats/cards', { params: { query: { version } } })
	]);
	return {
		cards: ok(cards),
		versions: ok(versions).map((v) => v.version),
		decks: ok(decks),
		cardStats: ok(cardStats),
		sort
	};
};
