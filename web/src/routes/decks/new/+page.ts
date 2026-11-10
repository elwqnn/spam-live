import { redirect } from '@sveltejs/kit';
import { api, loginHref } from '#lib';
import { newDeckData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, parent }) => {
	const { me } = await parent();
	if (!me) redirect(303, loginHref(url));
	return newDeckData(api(fetch, url.origin));
};
