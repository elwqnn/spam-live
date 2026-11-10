import { describe, expect, it } from 'vitest';
import { promote } from './promote';

describe('promote', () => {
	it('promotes from a win score of 0.5', () => {
		expect(promote([{ winScore: 0.4999 }, { winScore: 0.5 }]).map((p) => p.reasons)).toEqual([
			[],
			['winrate']
		]);
	});

	it('does not promote a deck with 0 games', () => {
		expect(promote([{ winScore: null }, {}])).toEqual([
			{ reasons: [], large: false },
			{ reasons: [], large: false }
		]);
	});

	it('promotes only the first 3 decks', () => {
		const decks = Array.from({ length: 5 }, () => ({ winScore: 0.6 }));
		expect(promote(decks).map((p) => p.reasons.length)).toEqual([1, 1, 1, 0, 0]);
	});

	it('makes only the first deck large, when it has a reason and the list has 3 decks or more', () => {
		const strong = { winScore: 0.7 };
		expect(promote([strong, strong, strong]).map((p) => p.large)).toEqual([true, false, false]);
		expect(promote([strong, strong]).map((p) => p.large)).toEqual([false, false]);
		expect(promote([{ winScore: 0.2 }, strong, strong]).map((p) => p.large)).toEqual([
			false,
			false,
			false
		]);
	});
});
