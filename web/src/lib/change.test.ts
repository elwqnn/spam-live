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
	it('gives text-up above 50 %', () => {
		expect(winRateColor(0.55)).toBe('text-up');
		expect(winRateColor(0.501)).toBe('text-up');
		expect(winRateColor(1)).toBe('text-up');
	});

	it('gives text-down below 50 %', () => {
		expect(winRateColor(0.45)).toBe('text-down');
		expect(winRateColor(0.499)).toBe('text-down');
		expect(winRateColor(0)).toBe('text-down');
	});

	it('gives no class when the shown value is 50 %', () => {
		expect(winRateColor(0.5)).toBeUndefined();
		expect(winRateColor(0.5004)).toBeUndefined();
		expect(winRateColor(0.4996)).toBeUndefined();
	});

	it('gives no class for no value', () => {
		expect(winRateColor(null)).toBeUndefined();
		expect(winRateColor(undefined)).toBeUndefined();
	});
});
