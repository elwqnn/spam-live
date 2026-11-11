import { redirect } from '@sveltejs/kit';
import { href } from '#lib/host.js';
import type { PageLoad } from './$types';

export const load: PageLoad = () => {
	redirect(307, href('/settings'));
};
