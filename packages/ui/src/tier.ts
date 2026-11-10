import type { Schemas } from './index.js';

export type Tier = Schemas['Tier'];

export const TIERS: Tier[] = ['S', 'A', 'B', 'C', 'D'];

type Ranked = Pick<Schemas['CardStats'], 'card' | 'games' | 'adjustedWinRate' | 'tier'>;

export const tierCards = <T extends Ranked>(stats: T[], tier: Tier) =>
	stats
		.filter((s) => s.tier === tier)
		.sort(
			(a, b) =>
				(b.adjustedWinRate ?? 0) - (a.adjustedWinRate ?? 0) ||
				b.games - a.games ||
				(a.card < b.card ? -1 : a.card > b.card ? 1 : 0)
		);
