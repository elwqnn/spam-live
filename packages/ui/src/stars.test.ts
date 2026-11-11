import { describe, expect, it } from 'vitest';
import { minStars } from './stars';

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
