import { api } from '#lib';
import { patchNotesData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ fetch, url, untrack }) =>
	patchNotesData(
		api(
			fetch,
			untrack(() => url.origin)
		)
	);
