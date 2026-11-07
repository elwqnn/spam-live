import { api, ok } from '#lib';
import { selection } from '#lib/filter.js';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

const TABS = ['meta', 'tiers', 'counters', 'duos', 'length', 'trends'] as const;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

export const load: PageLoad = async ({ fetch, url, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const tab = TABS.find((t) => t === url.searchParams.get('tab')) ?? 'meta';
	const version = url.searchParams.get('version') ?? undefined;
	const query = { params: { query: { version } } };
	const against = [
		...new Set(
			selection(url.searchParams.get('against'))
				.map((v) => v.toLowerCase())
				.filter((v) => UUID.test(v))
		)
	].slice(0, 4);
	const versioned = tab !== 'trends';
	const [cards, versions, overview, stats, counters, duos, pace, history] = await Promise.all([
		versioned
			? client.GET('/api/cards', { params: { query: { lang: getLocale() } } }).then(ok)
			: [],
		versioned ? client.GET('/api/catalog/versions').then(ok) : [],
		tab === 'meta' ? client.GET('/api/stats/overview', query).then(ok) : null,
		tab === 'meta' || tab === 'tiers' || tab === 'duos'
			? client.GET('/api/stats/cards', query).then(ok)
			: [],
		tab === 'counters' && against.length
			? client
					.GET('/api/stats/counters', {
						params: { query: { version, against: against.join(',') } }
					})
					.then(ok)
			: null,
		tab === 'duos' ? client.GET('/api/stats/duos', query).then(ok) : [],
		tab === 'length' ? client.GET('/api/stats/pace', query).then(ok) : null,
		tab === 'trends' ? client.GET('/api/stats/history').then(ok) : null
	]);
	return {
		tab,
		cards,
		versions: versions.map((v) => v.version),
		overview,
		stats,
		against,
		counters,
		duos,
		pace,
		history
	};
};
