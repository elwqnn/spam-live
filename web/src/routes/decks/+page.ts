import { api } from '#lib';
import { decksData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, parent, untrack, depends }) => {
	depends('app:decks');
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const { me } = await parent();
	return decksData(client, url, me);
};
