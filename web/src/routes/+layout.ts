import { api } from '#lib';
import { namesData } from '@spam-live/ui/views/data.js';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ fetch, url, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const [{ data: me }, names] = await Promise.all([client.GET('/api/me'), namesData(client)]);
	return { me: me ?? null, names };
};
