import { client } from '#lib/host.js';
import { deckData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params, depends }) => {
	depends('app:decks');
	return deckData(client(), Number(params.id));
};
