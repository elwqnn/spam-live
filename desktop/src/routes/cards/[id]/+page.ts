import { client } from '#lib/host.js';
import { cardData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ url, params, depends }) => {
	depends('app:decks');
	return cardData(client(), url, params.id);
};
