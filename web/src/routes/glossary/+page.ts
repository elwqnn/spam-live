import { api } from '#lib';
import { glossaryData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url, untrack }) =>
	glossaryData(
		api(
			fetch,
			untrack(() => url.origin)
		)
	);
