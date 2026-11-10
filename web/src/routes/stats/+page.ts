import { api } from '#lib';
import { statsData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url, untrack }) =>
	statsData(
		api(
			fetch,
			untrack(() => url.origin)
		),
		url
	);
