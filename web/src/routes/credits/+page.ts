import { api, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const artists = await client.GET('/api/artists', { params: { query: { lang: getLocale() } } });
	return { artists: ok(artists) };
};
