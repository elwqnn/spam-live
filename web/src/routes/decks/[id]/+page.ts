import { api } from '#lib';
import { deckData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url, params, untrack, depends }) => {
	depends('app:decks');
	return deckData(
		api(
			fetch,
			untrack(() => url.origin)
		),
		Number(params.id)
	);
};
