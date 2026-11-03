import { api, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, params, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const version = url.searchParams.get('version') ?? undefined;
	const [card, stats, versions] = await Promise.all([
		client.GET('/api/cards/{id}', {
			params: { path: { id: params.id }, query: { lang: getLocale() } }
		}),
		client.GET('/api/stats/cards', { params: { query: { version } } }),
		client.GET('/api/catalog/versions')
	]);
	return {
		card: ok(card),
		stats: ok(stats).find((s) => s.card === params.id) ?? null,
		versions: ok(versions).map((v) => v.version)
	};
};
