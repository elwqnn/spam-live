import { redirect } from '@sveltejs/kit';
import { client, href } from '#lib/host.js';
import { editDeckData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, parent }) => {
	const { me } = await parent();
	if (!me) redirect(307, href('/settings'));
	return editDeckData(client(), Number(params.id), me);
};
