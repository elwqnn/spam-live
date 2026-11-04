import { fold } from './description.js';

const WORD = /[\p{L}\p{N}]/u;

export function rank(name: string, query: string) {
	const text = fold(name);
	let best = -1;
	for (let i = text.indexOf(query); i >= 0; i = text.indexOf(query, i + 1)) {
		if (i === 0) return 0;
		if (!WORD.test(text[i - 1])) return 1;
		best = 2;
	}
	return best;
}

export function search<T>(
	items: readonly T[],
	query: string,
	name: (item: T) => string,
	limit = Infinity
): T[] {
	const q = fold(query.trim());
	if (!q) return [];
	return items
		.map((item) => ({ item, name: name(item), rank: rank(name(item), q) }))
		.filter((x) => x.rank >= 0)
		.sort((a, b) => a.rank - b.rank || a.name.length - b.name.length)
		.slice(0, limit)
		.map((x) => x.item);
}
