import { redirect } from '@sveltejs/kit';
import { api, loginHref } from '#lib';
import { editDeckData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, params, parent }) => {
	const { me } = await parent();
	if (!me) redirect(303, loginHref(url));
	return editDeckData(api(fetch, url.origin), Number(params.id), me);
};
