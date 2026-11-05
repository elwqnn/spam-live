import { api, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const query = { params: { query: { version: url.searchParams.get('version') ?? undefined } } };
	const [cards, versions, matchups, pace, history] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang: getLocale() } } }),
		client.GET('/api/catalog/versions'),
		client.GET('/api/stats/matchups', query),
		client.GET('/api/stats/pace', query),
		client.GET('/api/stats/history')
	]);
	return {
		cards: ok(cards),
		versions: ok(versions).map((v) => v.version),
		matchups: ok(matchups),
		pace: ok(pace),
		history: ok(history)
	};
};
