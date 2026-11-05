export const MIN_GAMES = 20;

export type Point = {
	label: string;
	games: number;
	wins: number;
	winRate: number | null;
};

type Result = { turns: number; games: number; wins: number };

export const bucketStart = (turns: number) => turns - ((turns - 1) % 3);

export function buckets<T extends { turns: Result[] }>(cards: T[]): (T & { points: Point[] })[] {
	const starts = cards.flatMap((c) =>
		c.turns.filter((t) => t.games > 0).map((t) => bucketStart(t.turns))
	);
	const first = Math.min(...starts);
	const range = starts.length
		? Array.from({ length: (Math.max(...starts) - first) / 3 + 1 }, (_, i) => first + i * 3)
		: [];
	const all = cards.map((c) =>
		range.map((start) => {
			const items = c.turns.filter((t) => bucketStart(t.turns) === start);
			const games = items.reduce((n, t) => n + t.games, 0);
			const wins = items.reduce((n, t) => n + t.wins, 0);
			return {
				label: `${start}–${start + 2}`,
				games,
				wins,
				winRate: games >= MIN_GAMES ? wins / games : null
			};
		})
	);
	const shown = (i: number) => all.some((points) => points[i].winRate != null);
	const from = range.findIndex((_, i) => shown(i));
	const to = range.findLastIndex((_, i) => shown(i));
	return cards.map((c, i) => ({ ...c, points: all[i].slice(from, to + 1) }));
}

export function rateDomain(values: number[]): [number, number] | undefined {
	if (!values.length) return undefined;
	return [
		Math.max(0, Math.floor(Math.min(...values) * 20 - 1)) / 20,
		Math.min(20, Math.ceil(Math.max(...values) * 20 + 1)) / 20
	];
}
