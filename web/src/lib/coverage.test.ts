import { describe, expect, it } from 'vitest';
import type { components } from './api.js';
import { coverage } from './coverage';

type CardBase = components['schemas']['CardBase'];

const card = (id: string, order: number, element: string, rarity: string): CardBase => ({
	id,
	order,
	element,
	rarity,
	hp: 10,
	goldIncome: 1,
	resistances: { earth: 0, fire: 0, light: 0, nature: 0, shadow: 0, water: 0 },
	face: { abilities: [], tox: [] }
});

const stat = (card: string, games: number) => ({ card, games, wins: 0, playRate: 0 });

const cards = [
	card('c', 3, 'water', 'rarity-epic'),
	card('a', 1, 'fire', 'rarity-common'),
	card('b', 2, 'fire', 'rarity-common'),
	card('d', 4, 'water', 'rarity-common')
];

describe('coverage', () => {
	it('counts the cards with at least 1 game', () => {
		const result = coverage(cards, [stat('a', 3), stat('c', 1), stat('d', 0), stat('x', 9)]);
		expect(result.played).toBe(2);
		expect(result.total).toBe(4);
		expect(result.share).toBe(0.5);
		expect(result.rarities).toEqual({
			'rarity-common': { played: 1, total: 3 },
			'rarity-epic': { played: 1, total: 1 }
		});
		expect(result.elements).toEqual({
			fire: { played: 1, total: 2 },
			water: { played: 1, total: 2 }
		});
		expect(result.unplayed.map((c) => c.id)).toEqual(['b', 'd']);
	});

	it('gives 0 without games and without cards', () => {
		expect(coverage(cards, []).played).toBe(0);
		expect(coverage([], []).share).toBe(0);
	});
});
