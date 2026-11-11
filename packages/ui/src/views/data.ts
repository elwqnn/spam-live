import { error } from '@sveltejs/kit';
import { ok, type Client, type Schemas } from '#ui';
import { selection } from '#ui/filter.js';
import { getLocale } from '$paraglide/runtime.js';

type Query = Record<string, string | null>;
export type Me = Schemas['Me'];

const DECKS_PAGE = 24;
const MATCHES_PAGE = 24;
const HISTORY_PAGE = 10;
const MAX_OFFSET = 10_000;
const SORTS = ['votes', 'recent', 'winrate'] as const;
const STATS_TABS = ['meta', 'counters', 'duos', 'length', 'trends'] as const;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const NO_MATCHUPS: Schemas['CardMatchups'] = { strong: [], weak: [] };

async function cardDetails(client: Client, ids: string[], lang: string) {
	return Promise.all(
		ids.map(async (id) => {
			const { data } = await client.GET('/api/cards/{id}', {
				params: { path: { id }, query: { lang } }
			});
			return data ?? null;
		})
	);
}

export async function cardsData(client: Client, url: URL) {
	const trend = url.searchParams.get('trend') === 'version' ? 'version' : 'week';
	const [cards, stats] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang: getLocale() } } }),
		client.GET('/api/stats/cards', { params: { query: { trend } } })
	]);
	return { cards: ok(cards), stats: stats.data ?? [] };
}

export async function cardData(client: Client, url: URL, id: string) {
	const lang = getLocale();
	const version = url.searchParams.get('version') ?? undefined;
	const trend =
		version !== 'all' && url.searchParams.get('trend') === 'version' ? 'version' : 'week';
	const [card, stats, versions, decks, keywords, found] = await Promise.all([
		client.GET('/api/cards/{id}', { params: { path: { id }, query: { lang } } }),
		client.GET('/api/stats/cards', { params: { query: { version, trend } } }),
		client.GET('/api/catalog/versions'),
		client.GET('/api/decks', { params: { query: { card: id, limit: 6 } } }),
		client.GET('/api/glossary/keywords', { params: { query: { lang } } }),
		client.GET('/api/stats/cards/{id}/matchups', {
			params: { path: { id }, query: { version } }
		})
	]);
	const cardStats = ok(stats);
	const list = decks.data ?? [];
	const matchups = found.data ?? NO_MATCHUPS;
	const cards =
		list.length || matchups.strong.length || matchups.weak.length
			? (await client.GET('/api/cards', { params: { query: { lang } } })).data
			: [];
	return {
		card: ok(card),
		stats: cardStats.find((s) => s.card === id) ?? null,
		cardStats,
		versions: ok(versions).map((v) => v.version),
		decks: cards ? list : [],
		matchups: cards ? matchups : NO_MATCHUPS,
		cards: cards ?? [],
		keywords: keywords.data ?? []
	};
}

export async function decksData(client: Client, url: URL, me: Me | null) {
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
					limit: DECKS_PAGE + 1
				}
			}
		})
	]);
	const list = ok(decks);
	const last = list[DECKS_PAGE - 1];
	const next: Query | null =
		list.length <= DECKS_PAGE || (sort !== 'recent' && offset + DECKS_PAGE > MAX_OFFSET)
			? null
			: sort === 'recent'
				? { before: last.publishedAt ?? last.createdAt }
				: { offset: String(offset + DECKS_PAGE) };
	const prev: Query | null =
		sort === 'recent'
			? before
				? { before: null }
				: null
			: offset > 0
				? { offset: offset > DECKS_PAGE ? String(offset - DECKS_PAGE) : null }
				: null;
	return {
		cards: ok(cards),
		decks: list.slice(0, DECKS_PAGE),
		sort,
		mine,
		card,
		next,
		prev
	};
}

export async function deckData(client: Client, id: number) {
	const lang = getLocale();
	const path = { params: { path: { id } } };
	const keywords = client.GET('/api/glossary/keywords', { params: { query: { lang } } });
	const deck = ok(await client.GET('/api/decks/{id}', path));
	const [comments, found, cards] = await Promise.all([
		deck.status === 'public'
			? client.GET('/api/decks/{id}/comments', path).then((r) => r.data)
			: [],
		client.GET('/api/decks/{id}/matchups', path).then((r) => r.data),
		cardDetails(client, deck.cards, lang)
	]);
	const matchups = found ?? NO_MATCHUPS;
	const others =
		matchups.strong.length || matchups.weak.length
			? (await client.GET('/api/cards', { params: { query: { lang } } })).data
			: [];
	return {
		deck,
		cards,
		comments: comments ?? [],
		matchups: others ? matchups : NO_MATCHUPS,
		others: others ?? [],
		keywords: (await keywords).data ?? []
	};
}

export async function newDeckData(client: Client) {
	const query = { params: { query: { lang: getLocale() } } };
	const [cards, keywords] = await Promise.all([
		client.GET('/api/cards', query),
		client.GET('/api/glossary/keywords', query)
	]);
	return { cards: ok(cards), keywords: keywords.data ?? [] };
}

export async function editDeckData(client: Client, id: number, me: Me) {
	const lang = getLocale();
	const [cards, deck, keywords] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang } } }),
		client.GET('/api/decks/{id}', { params: { path: { id } } }),
		client.GET('/api/glossary/keywords', { params: { query: { lang } } })
	]);
	const detail = ok(deck);
	if (detail.authorId !== me.id) error(404, 'Not Found');
	const details = await cardDetails(client, detail.cards, lang);
	return { cards: ok(cards), deck: detail, details, keywords: keywords.data ?? [] };
}

export async function statsData(client: Client, url: URL) {
	const tab = STATS_TABS.find((t) => t === url.searchParams.get('tab')) ?? 'meta';
	const version = url.searchParams.get('version') ?? undefined;
	const query = { params: { query: { version } } };
	const against = [
		...new Set(
			selection(url.searchParams.get('against'))
				.map((v) => v.toLowerCase())
				.filter((v) => UUID.test(v))
		)
	].slice(0, 4);
	const versioned = tab !== 'trends';
	const [cards, versions, overview, stats, counters, duos, pace, history] = await Promise.all([
		versioned
			? client.GET('/api/cards', { params: { query: { lang: getLocale() } } }).then(ok)
			: [],
		versioned ? client.GET('/api/catalog/versions').then(ok) : [],
		tab === 'meta' ? client.GET('/api/stats/overview', query).then(ok) : null,
		tab === 'meta' || tab === 'duos' ? client.GET('/api/stats/cards', query).then(ok) : [],
		tab === 'counters' && against.length
			? client
					.GET('/api/stats/counters', {
						params: { query: { version, against: against.join(',') } }
					})
					.then(ok)
			: null,
		tab === 'duos' ? client.GET('/api/stats/duos', query).then(ok) : [],
		tab === 'length' ? client.GET('/api/stats/pace', query).then(ok) : null,
		tab === 'trends' ? client.GET('/api/stats/history').then(ok) : null
	]);
	return {
		tab,
		cards,
		versions: versions.map((v) => v.version),
		overview,
		stats,
		against,
		counters,
		duos,
		pace,
		history
	};
}

export async function glossaryData(client: Client) {
	const query = { params: { query: { lang: getLocale() } } };
	const [glossary, cards] = await Promise.all([
		client.GET('/api/glossary', query),
		client.GET('/api/cards', query)
	]);
	return { glossary: ok(glossary), cards: ok(cards) };
}

export async function meData(client: Client, url: URL, me: Me) {
	const before = url.searchParams.get('before') || undefined;
	const [cards, matches, decks, devices] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang: getLocale() } } }),
		client.GET('/api/me/matches', { params: { query: { before, limit: MATCHES_PAGE + 1 } } }),
		client.GET('/api/decks', { params: { query: { author: 'me', sort: 'recent', limit: 6 } } }),
		client.GET('/api/me/devices')
	]);
	const list = ok(matches);
	return {
		me,
		cards: ok(cards),
		matches: list.slice(0, MATCHES_PAGE),
		next: list.length > MATCHES_PAGE ? list[MATCHES_PAGE - 1].startUtc : null,
		first: !before,
		decks: ok(decks),
		devices: ok(devices)
	};
}

export async function dashboardData(client: Client, url: URL, me: Me | null) {
	const before = url.searchParams.get('before') || undefined;
	const [cards, stats, matches, top] = await Promise.all([
		client.GET('/api/cards', { params: { query: { lang: getLocale() } } }).then(ok),
		client.GET('/api/stats/cards').then((r) => r.data ?? []),
		me
			? client
					.GET('/api/me/matches', { params: { query: { before, limit: HISTORY_PAGE + 1 } } })
					.then(ok)
			: [],
		me ? client.GET('/api/me/top-decks').then(ok) : []
	]);
	return {
		cards,
		stats,
		matches: matches.slice(0, HISTORY_PAGE),
		next: matches.length > HISTORY_PAGE ? matches[HISTORY_PAGE - 1].startUtc : null,
		first: !before,
		top
	};
}

export async function accountData(client: Client, me: Me) {
	return { me, devices: ok(await client.GET('/api/me/devices')) };
}

export async function creditsData(client: Client) {
	const artists = await client.GET('/api/artists', { params: { query: { lang: getLocale() } } });
	return { artists: ok(artists) };
}
