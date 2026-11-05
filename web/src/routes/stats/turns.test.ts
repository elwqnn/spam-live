import { describe, expect, it } from 'vitest';
import { bucketStart, buckets, rateDomain } from './turns';

describe('bucketStart', () => {
	it('groups turns by 3 from 1', () => {
		expect([1, 2, 3, 4, 6, 7, 9, 10].map(bucketStart)).toEqual([1, 1, 1, 4, 4, 7, 7, 10]);
	});
});

describe('buckets', () => {
	it('sums each card on 1 shared range of buckets with a win rate', () => {
		const result = buckets([
			{
				card: 'a',
				turns: [
					{ turns: 7, games: 15, wins: 9 },
					{ turns: 9, games: 10, wins: 4 },
					{ turns: 16, games: 3, wins: 1 }
				]
			},
			{
				card: 'b',
				turns: [
					{ turns: 4, games: 5, wins: 5 },
					{ turns: 13, games: 30, wins: 12 }
				]
			}
		]);
		expect(result.map((c) => c.points.map((p) => p.label))).toEqual([
			['7–9', '10–12', '13–15'],
			['7–9', '10–12', '13–15']
		]);
		expect(result[0].card).toBe('a');
		expect(result[0].points[0]).toEqual({ label: '7–9', games: 25, wins: 13, winRate: 13 / 25 });
		expect(result[0].points[1]).toEqual({ label: '10–12', games: 0, wins: 0, winRate: null });
		expect(result[1].points[0]).toEqual({ label: '7–9', games: 0, wins: 0, winRate: null });
		expect(result[1].points[2].winRate).toBe(0.4);
	});

	it('gives null under 20 games', () => {
		const [card] = buckets([
			{
				turns: [
					{ turns: 1, games: 20, wins: 10 },
					{ turns: 5, games: 19, wins: 19 },
					{ turns: 8, games: 20, wins: 5 }
				]
			}
		]);
		expect(card.points.map((p) => p.winRate)).toEqual([0.5, null, 0.25]);
	});

	it('handles an empty input', () => {
		expect(buckets([])).toEqual([]);
		expect(buckets([{ card: 'a', turns: [] }])).toEqual([{ card: 'a', turns: [], points: [] }]);
		const few = [{ card: 'a', turns: [{ turns: 4, games: 19, wins: 9 }] }];
		expect(buckets(few)).toEqual([{ ...few[0], points: [] }]);
	});
});

describe('rateDomain', () => {
	it('pads by 5 points and rounds out to 5 points', () => {
		expect(rateDomain([0.43, 0.57])).toEqual([0.35, 0.65]);
		expect(rateDomain([0.02, 0.99])).toEqual([0, 1]);
		expect(rateDomain([])).toBeUndefined();
	});
});
