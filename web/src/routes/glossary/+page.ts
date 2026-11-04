import { api, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const query = { params: { query: { lang: getLocale() } } };
	const [glossary, cards] = await Promise.all([
		client.GET('/api/glossary', query),
		client.GET('/api/cards', query)
	]);
	return { glossary: ok(glossary), cards: ok(cards) };
};
