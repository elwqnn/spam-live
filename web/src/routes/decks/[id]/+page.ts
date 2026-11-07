import { api, ok, type Schemas } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, params, untrack, depends }) => {
	depends('app:decks');
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const lang = getLocale();
	const path = { params: { path: { id: Number(params.id) } } };
	const keywords = client.GET('/api/glossary/keywords', { params: { query: { lang } } });
	const deck = ok(await client.GET('/api/decks/{id}', path));
	const [comments, found, ...cards] = await Promise.all([
		deck.status === 'public'
			? client.GET('/api/decks/{id}/comments', path).then((r) => r.data)
			: [],
		client.GET('/api/decks/{id}/matchups', path).then((r) => r.data),
		...deck.cards.map(async (id) => {
			const { data } = await client.GET('/api/cards/{id}', {
				params: { path: { id }, query: { lang } }
			});
			return data ?? null;
		})
	]);
	const none: Schemas['CardMatchups'] = { strong: [], weak: [] };
	const matchups = found ?? none;
	const others =
		matchups.strong.length || matchups.weak.length
			? (await client.GET('/api/cards', { params: { query: { lang } } })).data
			: [];
	return {
		deck,
		cards,
		comments: comments ?? [],
		matchups: others ? matchups : none,
		others: others ?? [],
		keywords: (await keywords).data ?? []
	};
};
