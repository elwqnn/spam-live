import { redirect } from '@sveltejs/kit';
import { start } from '#lib/app.js';
import { href } from '#lib/host.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ parent }) => {
	redirect(307, href(start((await parent()).me)));
};
