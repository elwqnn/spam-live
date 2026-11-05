import { api } from '#lib';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, url }) => {
	const res = await api(fetch, url.origin).GET('/api/desktop/release');
	return { release: res.data ?? null };
};
