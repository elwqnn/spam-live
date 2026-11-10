import { client } from '#lib/host.js';
import { glossaryData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = () => glossaryData(client());
