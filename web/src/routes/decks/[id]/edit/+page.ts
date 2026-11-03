import { error, redirect } from '@sveltejs/kit';
import { api, loginHref, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, params, parent }) => {
	const { me } = await parent();
	if (!me) redirect(303, loginHref(url));
	const client = api(fetch, url.origin);
	const lang = getLocale();
	const [cards, deck] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang } } }),
		client.GET('/api/decks/{id}', { params: { path: { id: Number(params.id) } } })
	]);
	const detail = ok(deck);
	if (detail.authorId !== me.id) error(404, 'Not Found');
	const details = await Promise.all(
		detail.cards.map(async (id) => {
			const { data } = await client.GET('/api/cards/{id}', {
				params: { path: { id }, query: { lang } }
			});
			return data ?? null;
		})
	);
	return { cards: ok(cards), deck: detail, details };
};
