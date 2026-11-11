import { client } from '#lib/host.js';
import { dashboardData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url, parent, depends }) => {
	depends('app:decks');
	const { me } = await parent();
	return dashboardData(client(), url, me);
};
