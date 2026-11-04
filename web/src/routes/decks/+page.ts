import { api, ok } from '#lib';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

const PAGE_SIZE = 24;
const MAX_OFFSET = 10_000;
const SORTS = ['votes', 'recent', 'winrate'] as const;
type Query = Record<string, string | null>;

export const load: PageLoad = async ({ fetch, url, parent, untrack }) => {
	const client = api(
		fetch,
		untrack(() => url.origin)
	);
	const { me } = await parent();
	const query = url.searchParams;
	const sort = SORTS.find((s) => s === query.get('sort')) ?? 'votes';
	const mine = !!me && query.get('tab') === 'mine';
	const offset =
		sort === 'recent'
			? 0
			: Math.min(MAX_OFFSET, Math.max(0, Math.floor(Number(query.get('offset')) || 0)));
	const before = (sort === 'recent' && query.get('before')) || undefined;
	const card = query.get('card') || undefined;
	const q = query.get('q')?.trim() ?? '';
	const search = [...q].length >= 2 && [...q].length <= 64 ? q : undefined;
	const [cards, decks] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang: getLocale() } } }),
		client.GET('/api/decks', {
			params: {
				query: {
					sort,
					card,
					offset: offset || undefined,
					before,
					author: mine ? 'me' : undefined,
					q: search,
					limit: PAGE_SIZE + 1
				}
			}
		})
	]);
	const list = ok(decks);
	const last = list[PAGE_SIZE - 1];
	const next: Query | null =
		list.length <= PAGE_SIZE || (sort !== 'recent' && offset + PAGE_SIZE > MAX_OFFSET)
			? null
			: sort === 'recent'
				? { before: last.publishedAt ?? last.createdAt }
				: { offset: String(offset + PAGE_SIZE) };
	const prev: Query | null =
		sort === 'recent'
			? before
				? { before: null }
				: null
			: offset > 0
				? { offset: offset > PAGE_SIZE ? String(offset - PAGE_SIZE) : null }
				: null;
	return {
		cards: ok(cards),
		decks: list.slice(0, PAGE_SIZE),
		sort,
		mine,
		card,
		next,
		prev
	};
};
