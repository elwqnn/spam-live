import { redirect } from '@sveltejs/kit';
import { api, loginHref, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, parent }) => {
	const { me } = await parent();
	if (!me) redirect(303, loginHref(url));
	const cards = await api(fetch, url.origin).GET('/api/cards', {
		params: { query: { lang: getLocale() } }
	});
	return { cards: ok(cards) };
};
