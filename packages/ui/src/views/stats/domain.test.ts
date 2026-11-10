import { describe, expect, it } from 'vitest';
import { rateDomain } from './domain';

describe('rateDomain', () => {
	it('pads by 5 points and rounds out to 5 points', () => {
		expect(rateDomain([0.43, 0.57])).toEqual([0.35, 0.65]);
		expect(rateDomain([0.02, 0.99])).toEqual([0, 1]);
		expect(rateDomain([])).toBeUndefined();
	});
});
