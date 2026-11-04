import { describe, expect, it } from 'vitest';
import { selection, toggle } from './filter';

describe('selection', () => {
	it('reads comma-separated values and old single values', () => {
		expect(selection('fire,water')).toEqual(['fire', 'water']);
		expect(selection('fire')).toEqual(['fire']);
		expect(selection('fire,,')).toEqual(['fire']);
		expect(selection('')).toEqual([]);
		expect(selection(null)).toEqual([]);
	});
});

describe('toggle', () => {
	it('adds a value that is not selected', () => {
		expect(toggle([], 'fire')).toBe('fire');
		expect(toggle(['fire'], 'water')).toBe('fire,water');
	});

	it('removes a selected value', () => {
		expect(toggle(['fire', 'water'], 'fire')).toBe('water');
	});

	it('returns null when no value stays', () => {
		expect(toggle(['fire'], 'fire')).toBeNull();
	});
});
