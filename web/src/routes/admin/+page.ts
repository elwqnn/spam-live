import { error } from '@sveltejs/kit';
import { api, ok } from '#lib';
import { RANGES } from '#lib/analytics.js';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

const TABS = ['reports', 'users', 'matches', 'cards', 'analytics'] as const;

export const load: PageLoad = async ({ fetch, url, parent }) => {
	const { me } = await parent();
	if (!me?.isAdmin) error(404, 'Not Found');
	const client = api(fetch, url.origin);
	const tab = TABS.find((t) => t === url.searchParams.get('tab')) ?? 'reports';
	const range = RANGES.find((r) => r === url.searchParams.get('range')) ?? '7d';
	const [reports, disputed, unknown, cards, analytics] = await Promise.all([
		tab === 'reports'
			? client.GET('/api/admin/reports', { params: { query: { status: 'open' } } }).then(ok)
			: [],
		tab === 'matches' ? client.GET('/api/admin/matches/disputed').then(ok) : [],
		tab === 'cards' ? client.GET('/api/admin/unknown-cards').then(ok) : [],
		tab === 'matches'
			? client.GET('/api/cards', { params: { query: { lang: getLocale() } } }).then(ok)
			: [],
		tab === 'analytics'
			? client.GET('/api/admin/analytics', { params: { query: { range } } }).then(ok)
			: null
	]);
	return { tab, reports, disputed, unknown, cards, analytics, range };
};
