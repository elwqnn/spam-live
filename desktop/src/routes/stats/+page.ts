import { client } from '#lib/host.js';
import { statsData } from '@spam-live/ui/views/data.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ url }) => statsData(client(), url);
