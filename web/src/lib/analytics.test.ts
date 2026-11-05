import { describe, expect, it } from 'vitest';
import { hourLabel, hourTicks } from './analytics';

const hours = (from: string, n: number) =>
	Array.from({ length: n }, (_, i) => new Date(Date.parse(from) + i * 3_600_000));

describe('hourTicks', () => {
	it('keeps each 4th UTC hour for 24h', () => {
		const ticks = hourTicks(hours('2026-11-04T13:00:00Z', 24), '24h');
		expect(ticks.map((d) => d.getUTCHours())).toEqual([16, 20, 0, 4, 8, 12]);
	});

	it('keeps each UTC midnight for 7d', () => {
		const ticks = hourTicks(hours('2026-10-29T13:00:00Z', 168), '7d');
		expect(ticks.map((d) => d.toISOString().slice(0, 10))).toEqual([
			'2026-10-30',
			'2026-10-31',
			'2026-11-01',
			'2026-11-02',
			'2026-11-03',
			'2026-11-04',
			'2026-11-05'
		]);
	});

	it('keeps each 5th UTC midnight for 30d', () => {
		const ticks = hourTicks(hours('2026-10-06T13:00:00Z', 720), '30d');
		expect(ticks).toHaveLength(6);
		expect(ticks.every((d) => d.getUTCHours() === 0)).toBe(true);
		expect(ticks[1].getTime() - ticks[0].getTime()).toBe(5 * 24 * 3_600_000);
	});

	it('makes the step longer to keep at most max ticks', () => {
		const day = hourTicks(hours('2026-11-04T13:00:00Z', 24), '24h', 3);
		expect(day.map((d) => d.getUTCHours())).toEqual([16, 0, 8]);
		const week = hourTicks(hours('2026-10-29T13:00:00Z', 168), '7d', 3);
		expect(week.length).toBeLessThanOrEqual(3);
		expect(week[1].getTime() - week[0].getTime()).toBe(3 * 24 * 3_600_000);
		const month = hourTicks(hours('2026-10-06T13:00:00Z', 720), '30d', 3);
		expect(month.length).toBeLessThanOrEqual(3);
		expect(month[1].getTime() - month[0].getTime()).toBe(10 * 24 * 3_600_000);
		expect(hourTicks(hours('2026-10-29T13:00:00Z', 168), '7d', 16)).toHaveLength(7);
	});
});

describe('hourLabel', () => {
	it('formats in UTC', () => {
		const d = new Date('2026-11-04T23:00:00Z');
		expect(hourLabel('24h', 'en')(d)).toBe('11 PM');
		expect(hourLabel('24h', 'fr')(d)).toBe('23 h');
		expect(hourLabel('7d', 'fr')(d)).toBe('4 nov.');
	});
});
