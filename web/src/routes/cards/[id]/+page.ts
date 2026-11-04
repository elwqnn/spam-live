import { api, ok } from '#lib';
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
	const [card, stats, versions, decks, keywords] = await Promise.all([
		client.GET('/api/cards/{id}', { params: { path: { id: params.id }, query: { lang } } }),
		client.GET('/api/stats/cards', { params: { query: { version, trend } } }),
		client.GET('/api/catalog/versions'),
		client.GET('/api/decks', { params: { query: { card: params.id, limit: 6 } } }),
		client.GET('/api/glossary/keywords', { params: { query: { lang } } })
	]);
	const list = decks.data ?? [];
	const cards = list.length
		? (await client.GET('/api/cards', { params: { query: { lang } } })).data
		: [];
	return {
		card: ok(card),
		stats: ok(stats).find((s) => s.card === params.id) ?? null,
		versions: ok(versions).map((v) => v.version),
		decks: cards ? list : [],
		cards: cards ?? [],
		keywords: keywords.data ?? []
	};
};
