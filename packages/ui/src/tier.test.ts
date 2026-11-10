import { describe, expect, it } from 'vitest';
import { tierCards } from './tier';

const row = (card: string, games: number, adjustedWinRate: number, tier: 'S' | 'A' | null) => ({
	card,
	games,
	adjustedWinRate,
	tier
});

describe('tierCards', () => {
	it('keeps the tier and sorts by adjusted win rate, games, then card', () => {
		const stats = [
			row('c', 40, 0.6, 'S'),
			row('b', 40, 0.6, 'S'),
			row('x', 90, 0.7, 'A'),
			row('d', 50, 0.6, 'S'),
			row('a', 20, 0.65, 'S'),
			row('y', 10, 0.9, null)
		];
		expect(tierCards(stats, 'S').map((s) => s.card)).toEqual(['a', 'd', 'b', 'c']);
		expect(tierCards(stats, 'A').map((s) => s.card)).toEqual(['x']);
	});
});
