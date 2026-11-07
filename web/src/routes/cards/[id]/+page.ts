import { api, ok, type Schemas } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, params, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const lang = getLocale();
	const version = url.searchParams.get('version') ?? undefined;
	const trend =
		version !== 'all' && url.searchParams.get('trend') === 'version' ? 'version' : 'week';
	const [card, stats, versions, decks, keywords, found] = await Promise.all([
		client.GET('/api/cards/{id}', { params: { path: { id: params.id }, query: { lang } } }),
		client.GET('/api/stats/cards', { params: { query: { version, trend } } }),
		client.GET('/api/catalog/versions'),
		client.GET('/api/decks', { params: { query: { card: params.id, limit: 6 } } }),
		client.GET('/api/glossary/keywords', { params: { query: { lang } } }),
		client.GET('/api/stats/cards/{id}/matchups', {
			params: { path: { id: params.id }, query: { version } }
		})
	]);
	const cardStats = ok(stats);
	const list = decks.data ?? [];
	const none: Schemas['CardMatchups'] = { strong: [], weak: [] };
	const matchups = found.data ?? none;
	const cards =
		list.length || matchups.strong.length || matchups.weak.length
			? (await client.GET('/api/cards', { params: { query: { lang } } })).data
			: [];
	return {
		card: ok(card),
		stats: cardStats.find((s) => s.card === params.id) ?? null,
		cardStats,
		versions: ok(versions).map((v) => v.version),
		decks: cards ? list : [],
		matchups: cards ? matchups : none,
		cards: cards ?? [],
		keywords: keywords.data ?? []
	};
};
