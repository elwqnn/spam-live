import { client } from '#lib/host.js';
import { namesData } from '@spam-live/ui/views/data.js';
import type { LayoutLoad } from './$types';

export const ssr = false;

export const load: LayoutLoad = async () => {
	const names = namesData(client());
	try {
		const { data, response } = await client().GET('/api/me');
		return { me: data ?? null, meFailed: !data && response.status !== 401, names: await names };
	} catch {
		return { me: null, meFailed: true, names: await names };
	}
};
