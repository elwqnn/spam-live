import { redirect } from '@sveltejs/kit';
import { client, href } from '#lib/host.js';
import { meData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url, parent, depends }) => {
	depends('app:decks');
	const { me } = await parent();
	if (!me) redirect(307, href('/settings'));
	return meData(client(), url, me);
};
