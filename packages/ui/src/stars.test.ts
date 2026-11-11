import { describe, expect, it } from 'vitest';
import { minStars, starLabel } from './stars';

describe('minStars', () => {
	it('reads a bracket', () => {
		expect(minStars('0')).toBe(0);
		expect(minStars('15')).toBe(15);
		expect(minStars('90')).toBe(90);
	});

	it('gives no filter for a missing, invalid or negative value', () => {
		for (const param of [null, '', 'all', 'abc', '-15', '20', '15.5', '015', ' 15']) {
			expect(minStars(param)).toBeUndefined();
		}
	});
});

describe('starLabel', () => {
	const ranks = [
		{ threshold: 0, name: 'No rank' },
		{ threshold: 45, name: 'Gold' }
	];

	it('shows the rank name of the threshold', () => {
		expect(starLabel(45, ranks)).toBe('Gold+');
		expect(starLabel(0, ranks)).toBe('No rank+');
	});

	it('shows the number without a rank for the threshold', () => {
		expect(starLabel(30, ranks)).toBe('30+ stars');
		expect(starLabel(45)).toBe('45+ stars');
	});
});
