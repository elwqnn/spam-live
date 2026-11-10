import { describe, expect, it } from 'vitest';
import { formatChange, winRateColor } from './change';

describe('formatChange', () => {
	it('shows the points with 1 decimal', () => {
		expect(formatChange(0.022, 'en')).toEqual({ sign: 1, value: '2.2', text: '2.2 pts' });
		expect(formatChange(-0.1234, 'en')).toEqual({ sign: -1, value: '12.3', text: '12.3 pts' });
	});

	it('uses the locale', () => {
		expect(formatChange(-0.022, 'fr')).toEqual({ sign: -1, value: '2,2', text: '2,2 pts' });
	});

	it('gives sign 0 for a change that rounds to 0.0', () => {
		expect(formatChange(0, 'en')).toEqual({ sign: 0, value: '0.0', text: '0.0 pts' });
		expect(formatChange(0.0004, 'en')).toEqual({ sign: 0, value: '0.0', text: '0.0 pts' });
		expect(formatChange(-0.0004, 'fr')).toEqual({ sign: 0, value: '0,0', text: '0,0 pts' });
	});

	it('gives nothing for null', () => {
		expect(formatChange(null, 'en')).toBeNull();
		expect(formatChange(undefined, 'fr')).toBeNull();
	});
});

describe('winRateColor', () => {
	const up = (weight: number) => `color-mix(in oklab, var(--up) ${weight}%, var(--foreground))`;
	const down = (weight: number) => `color-mix(in oklab, var(--down) ${weight}%, var(--foreground))`;

	it('mixes --up by the distance above 50 %', () => {
		expect(winRateColor(0.51)).toBe(up(10));
		expect(winRateColor(0.55)).toBe(up(50));
	});

	it('mixes --down by the distance below 50 %', () => {
		expect(winRateColor(0.45)).toBe(down(50));
	});

	it('gives the full token from 10 points away', () => {
		expect(winRateColor(0.6)).toBe(up(100));
		expect(winRateColor(0.75)).toBe(up(100));
		expect(winRateColor(0.4)).toBe(down(100));
		expect(winRateColor(0.2)).toBe(down(100));
	});

	it('gives no color when the shown value is 50 %', () => {
		expect(winRateColor(0.5)).toBeUndefined();
		expect(winRateColor(0.50004)).toBeUndefined();
		expect(winRateColor(0.4996)).toBeUndefined();
	});

	it('gives no color for no value', () => {
		expect(winRateColor(null)).toBeUndefined();
		expect(winRateColor(undefined)).toBeUndefined();
	});
});
