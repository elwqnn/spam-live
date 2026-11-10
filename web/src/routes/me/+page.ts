import { redirect } from '@sveltejs/kit';
import { api, loginHref } from '#lib';
import { meData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, parent, depends }) => {
	depends('app:decks');
	const { me } = await parent();
	if (!me) redirect(303, loginHref(url));
	return meData(api(fetch, url.origin), url, me);
};
