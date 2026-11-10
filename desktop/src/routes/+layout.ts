import { client } from '#lib/host.js';
import type { LayoutLoad } from './$types';

export const ssr = false;

export const load: LayoutLoad = async () => {
	try {
		const { data, response } = await client().GET('/api/me');
		return { me: data ?? null, meFailed: !data && response.status !== 401 };
	} catch {
		return { me: null, meFailed: true };
	}
};
