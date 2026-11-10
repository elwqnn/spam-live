import { client } from '#lib/host.js';
import { cardsData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ url }) => cardsData(client(), url);
