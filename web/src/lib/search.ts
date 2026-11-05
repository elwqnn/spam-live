import { fold } from './description.js';

const WORD = /[\p{L}\p{N}]/u;

function locate(text: string, query: string): [number, number] {
	let best: [number, number] = [-1, -1];
	for (let i = text.indexOf(query); i >= 0; i = text.indexOf(query, i + 1)) {
		if (i === 0) return [0, 0];
		if (!WORD.test(text[i - 1])) return [1, i];
		if (best[0] < 0) best = [2, i];
	}
	return best;
}

export const rank = (name: string, query: string) => locate(fold(name), query)[0];

export function highlight(name: string, query: string): [string, string, string] {
	const q = fold(query.trim());
	const chars = [...name];
	const at: number[] = [];
	let text = '';
	chars.forEach((c, i) => {
		const f = fold(c);
		text += f;
		at.push(...Array(f.length).fill(i));
	});
	const [found, i] = locate(text, q);
	if (!q || found < 0) return [name, '', ''];
	const start = at[i];
	const end = at[i + q.length - 1] + 1;
	const part = (from: number, to?: number) => chars.slice(from, to).join('');
	return [part(0, start), part(start, end), part(end)];
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
