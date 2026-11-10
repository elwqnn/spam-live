import { api } from '#lib';
import { creditsData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url, untrack }) =>
	creditsData(
		api(
			fetch,
			untrack(() => url.origin)
		)
	);
