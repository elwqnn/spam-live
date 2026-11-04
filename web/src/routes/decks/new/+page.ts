import { redirect } from '@sveltejs/kit';
import { api, loginHref, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, parent }) => {
	const { me } = await parent();
	if (!me) redirect(303, loginHref(url));
	const client = api(fetch, url.origin);
	const query = { params: { query: { lang: getLocale() } } };
	const [cards, keywords] = await Promise.all([
		client.GET('/api/cards', query),
		client.GET('/api/glossary/keywords', query)
	]);
	return { cards: ok(cards), keywords: keywords.data ?? [] };
};
