import { error } from '@sveltejs/kit';
import { api, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

const TABS = ['reports', 'users', 'matches', 'cards'] as const;

export const load: PageLoad = async ({ fetch, url, parent }) => {
	const { me } = await parent();
	if (!me?.isAdmin) error(404, 'Not Found');
	const client = api(fetch, url.origin);
	const tab = TABS.find((t) => t === url.searchParams.get('tab')) ?? 'reports';
	const [reports, disputed, unknown, cards] = await Promise.all([
		tab === 'reports'
			? client.GET('/api/admin/reports', { params: { query: { status: 'open' } } }).then(ok)
			: [],
		tab === 'matches' ? client.GET('/api/admin/matches/disputed').then(ok) : [],
		tab === 'cards' ? client.GET('/api/admin/unknown-cards').then(ok) : [],
		tab === 'matches'
			? client.GET('/api/cards', { params: { query: { lang: getLocale() } } }).then(ok)
			: []
	]);
	return { tab, reports, disputed, unknown, cards };
};
