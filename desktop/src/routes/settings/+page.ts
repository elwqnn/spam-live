import { client } from '#lib/host.js';
import { accountData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ parent }) => {
	const { me } = await parent();
	return { account: me ? await accountData(client(), me) : null };
};
