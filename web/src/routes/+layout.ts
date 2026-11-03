import { api } from '#lib';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ fetch, url, untrack }) => {
	const { data: me } = await api(
		fetch,
		untrack(() => url.origin)
	).GET('/api/me');
	return { me: me ?? null };
};
