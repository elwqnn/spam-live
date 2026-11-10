import { api } from '#lib';
import { cardsData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url, untrack }) =>
	cardsData(
		api(
			fetch,
			untrack(() => url.origin)
		),
		url
	);
