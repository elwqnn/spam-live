import { describe, expect, it } from 'vitest';
import { curveRates, fullWeeks, monday, spanRange, sparkPath } from './spark';

describe('spanRange', () => {
	it('keeps a wide range', () => {
		expect(spanRange([0.1, null, 0.3], 0.02)).toEqual([0.1, 0.3]);
	});

	it('widens a narrow range around its center', () => {
		const [lo, hi] = spanRange([0.5, 0.51], 0.04);
		expect(lo).toBeCloseTo(0.485);
		expect(hi).toBeCloseTo(0.525);
	});

	it('centers an empty row on 50 %', () => {
		expect(spanRange([null, null], 0.04)).toEqual([0.48, 0.52]);
	});
});

describe('sparkPath', () => {
	it('draws a line and breaks it at a gap', () => {
		expect(sparkPath([0, 1, null, 0.5, 0.5], [0, 1], 40, 10)).toBe('M0 10L10 0M30 5L40 5');
	});

	it('clamps values to the range', () => {
		expect(sparkPath([-1, 2], [0, 1], 10, 10)).toBe('M0 10L10 0');
	});

	it('draws a single point as a dot', () => {
		expect(sparkPath([null, 0.5, null], [0, 1], 20, 10)).toBe('M10 5h0');
	});
});

describe('curveRates', () => {
	it('fills missing groups, keeps small groups and gives null at 0 games', () => {
		const curve = [
			{ start: 4, games: 20, wins: 10 },
			{ start: 10, games: 9, wins: 9 },
			{ start: 13, games: 10, wins: 3 },
			{ start: 16, games: 0, wins: 0 }
		];
		expect(curveRates(curve)).toEqual([0.5, null, 1, 0.3, null]);
		expect(curveRates([])).toEqual([]);
	});
});

describe('fullWeeks', () => {
	const now = new Date('2026-11-05T12:00:00Z');

	it('finds the Monday of a UTC date', () => {
		expect(monday(now).toISOString().slice(0, 10)).toBe('2026-11-02');
		expect(monday(new Date('2026-11-01T23:59:00Z')).toISOString().slice(0, 10)).toBe('2026-10-26');
	});

	it('skips the partial current week', () => {
		const weeks = ['2026-10-12', '2026-10-19', '2026-10-26', '2026-11-02'];
		expect(fullWeeks(weeks, now, 3)).toEqual({
			last: '2026-10-26',
			before: '2026-10-19',
			recent: ['2026-10-12', '2026-10-19', '2026-10-26']
		});
	});

	it('starts at the first full week with data when there are fewer weeks than the maximum', () => {
		const weeks = ['2026-10-05', '2026-10-19', '2026-10-26', '2026-11-02'];
		expect(fullWeeks(weeks, now, 12)).toEqual({
			last: '2026-10-26',
			before: '2026-10-19',
			recent: ['2026-10-05', '2026-10-12', '2026-10-19', '2026-10-26']
		});
	});

	it('keeps the last weeks up to the maximum', () => {
		const weeks = ['2026-09-07', '2026-09-14', '2026-09-21', '2026-09-28', '2026-10-05'];
		expect(fullWeeks(weeks, now, 3)?.recent).toEqual(['2026-09-21', '2026-09-28', '2026-10-05']);
	});

	it('needs 2 full weeks', () => {
		expect(fullWeeks(['2026-10-26', '2026-11-02'], now, 8)).toBeNull();
		expect(fullWeeks([], now, 8)).toBeNull();
	});

	it('compares with the week 7 days before, also when it has no data', () => {
		expect(fullWeeks(['2026-10-12', '2026-10-26'], now, 2)).toEqual({
			last: '2026-10-26',
			before: '2026-10-19',
			recent: ['2026-10-19', '2026-10-26']
		});
	});
});
