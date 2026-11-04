import { describe, expect, it } from 'vitest';
import { rank, search } from './search';

const id = (s: string) => s;

describe('rank', () => {
	it('ranks a name prefix, then a word prefix, then a substring', () => {
		expect(rank('Taunt', 'tau')).toBe(0);
		expect(rank('Big Taunt', 'tau')).toBe(1);
		expect(rank("L'Étau", 'etau')).toBe(1);
		expect(rank('Restaurant', 'tau')).toBe(2);
		expect(rank('Fire', 'tau')).toBe(-1);
	});

	it('finds a word prefix after an earlier substring', () => {
		expect(rank('Restaurant Taunt', 'tau')).toBe(1);
	});
});

describe('search', () => {
	it('folds case and accents', () => {
		expect(search(['Éclair', 'Fire'], 'ECL', id)).toEqual(['Éclair']);
		expect(search(['Eclair'], 'écl', id)).toEqual(['Eclair']);
	});

	it('sorts by rank, then by name length, and keeps the order of equal items', () => {
		expect(search(['Restaurant', 'Big Taunt', 'Taunting', 'Taunt', 'Tauz'], 'tau', id)).toEqual([
			'Tauz',
			'Taunt',
			'Taunting',
			'Big Taunt',
			'Restaurant'
		]);
	});

	it('applies the limit', () => {
		expect(search(['a1', 'a2', 'a3'], 'a', id, 2)).toEqual(['a1', 'a2']);
	});

	it('returns nothing for an empty query', () => {
		expect(search(['a'], '  ', id)).toEqual([]);
	});

	it('trims the query and reads the name with the callback', () => {
		expect(search([{ name: 'Taunt' }], ' tau ', (x) => x.name)).toEqual([{ name: 'Taunt' }]);
	});
});
