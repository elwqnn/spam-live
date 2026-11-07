import { api, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, params, untrack, depends }) => {
	depends('app:decks');
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const path = { params: { path: { id: Number(params.id) } } };
	const keywords = client.GET('/api/glossary/keywords', {
		params: { query: { lang: getLocale() } }
	});
	const deck = ok(await client.GET('/api/decks/{id}', path));
	const [comments, ...cards] = await Promise.all([
		deck.status === 'public'
			? client.GET('/api/decks/{id}/comments', path).then((r) => r.data)
			: [],
		...deck.cards.map(async (id) => {
			const { data } = await client.GET('/api/cards/{id}', {
				params: { path: { id }, query: { lang: getLocale() } }
			});
			return data ?? null;
		})
	]);
	return { deck, cards, comments: comments ?? [], keywords: (await keywords).data ?? [] };
};
