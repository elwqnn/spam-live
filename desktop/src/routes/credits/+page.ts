import { client } from '#lib/host.js';
import { creditsData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = () => creditsData(client());
