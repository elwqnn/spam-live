import { api, ok } from '#lib';
import { selection } from '#lib/filter.js';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

export const load: PageLoad = async ({ fetch, url, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const version = url.searchParams.get('version') ?? undefined;
	const query = { params: { query: { version } } };
	const against = [
		...new Set(
			selection(url.searchParams.get('against'))
				.map((v) => v.toLowerCase())
				.filter((v) => UUID.test(v))
		)
	].slice(0, 4);
	const [cards, versions, overview, stats, matchups, counters, duos, pace, turns, history] =
		await Promise.all([
			client.GET('/api/cards', { params: { query: { lang: getLocale() } } }),
			client.GET('/api/catalog/versions'),
			client.GET('/api/stats/overview', query),
			client.GET('/api/stats/cards', query),
			client.GET('/api/stats/matchups', query),
			against.length
				? client.GET('/api/stats/counters', {
						params: { query: { version, against: against.join(',') } }
					})
				: undefined,
			client.GET('/api/stats/duos', query),
			client.GET('/api/stats/pace', query),
			client.GET('/api/stats/turns', query),
			client.GET('/api/stats/history')
		]);
	return {
		cards: ok(cards),
		versions: ok(versions).map((v) => v.version),
		overview: ok(overview),
		stats: ok(stats),
		matchups: ok(matchups),
		against,
		counters: counters ? ok(counters) : null,
		duos: ok(duos),
		pace: ok(pace),
		turns: ok(turns),
		history: ok(history)
	};
};
