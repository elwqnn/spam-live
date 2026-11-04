import type { components } from './api.js';

type CardBase = components['schemas']['CardBase'];
type CardStats = components['schemas']['CardStats'];
export type Count = { played: number; total: number };

export function coverage(cards: CardBase[], stats: CardStats[]) {
	const games = new Map(stats.map((s) => [s.card, s.games]));
	const rarities: Record<string, Count> = {};
	const elements: Record<string, Count> = {};
	const unplayed: CardBase[] = [];
	for (const card of cards.toSorted((a, b) => a.order - b.order)) {
		const played = (games.get(card.id) ?? 0) > 0;
		if (!played) unplayed.push(card);
		for (const [group, key] of [
			[rarities, card.rarity],
			[elements, card.element]
		] as const) {
			group[key] ??= { played: 0, total: 0 };
			group[key].total++;
			if (played) group[key].played++;
		}
	}
	const total = cards.length;
	const played = total - unplayed.length;
	return { played, total, share: total ? played / total : 0, rarities, elements, unplayed };
}
