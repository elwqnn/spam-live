import { api } from '#lib';
import { cardData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url, params, untrack, depends }) => {
	depends('app:decks');
	return cardData(
		api(
			fetch,
			untrack(() => url.origin)
		),
		url,
		params.id
	);
};
