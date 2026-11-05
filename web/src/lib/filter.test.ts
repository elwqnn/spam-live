import { describe, expect, it } from 'vitest';
import { selection, target, toggle, type Filter } from './filter';

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

describe('target', () => {
	const multi: Filter = { id: 'element', label: 'Element', multiple: true, options: [] };
	const single: Filter = { id: 'sort', label: 'Sort', default: 'order', options: [] };

	it('toggles a value of a multiple filter', () => {
		expect(target(multi, ['fire'], 'water')).toBe('fire,water');
		expect(target(multi, ['fire', 'water'], 'fire')).toBe('water');
		expect(target(multi, ['fire'], 'fire')).toBeNull();
	});

	it('gives the value of a single filter', () => {
		expect(target(single, [], 'name')).toBe('name');
		expect(target(single, ['name'], 'hp')).toBe('hp');
		expect(target(single, ['name'], 'name')).toBe('name');
	});

	it('gives null for the default of a single filter', () => {
		expect(target(single, ['name'], 'order')).toBeNull();
		expect(target(single, [], 'order')).toBeNull();
	});
});
