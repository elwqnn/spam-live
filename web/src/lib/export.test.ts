import { describe, expect, it } from 'vitest';
import { BADGE, SIZES, credit, fileName, slug, tierLayout } from './export';

const height = (counts: number[], width: number, size: number, gap: number) => {
	const line = Math.max(1, Math.floor((width - BADGE) / (size + gap)));
	return counts.reduce(
		(sum, n) =>
			sum + Math.max(BADGE, Math.max(1, Math.ceil(n / line)) * (size + gap) - gap) + 2 * gap + 1,
		0
	);
};

describe('fileName', () => {
	it('joins the kind, the slug and the pixel size', () => {
		expect(fileName('card', slug('Élan Vital!'), SIZES[1])).toBe(
			'spam-live-card-elan-vital-1080x1920.png'
		);
	});

	it('uses the next text when a slug is empty', () => {
		expect(slug('???', '42')).toBe('42');
		expect(slug('1.29')).toBe('1-29');
		expect(slug('')).toBe('image');
	});
});

describe('credit', () => {
	it('names each artist 1 time', () => {
		expect(credit(['Ann', null, 'Bob', 'Ann'])).toBe('Illustrations: Ann and Bob');
		expect(credit(['Ann', 'Ann'])).toBe('Illustration: Ann');
		expect(credit([undefined])).toBeUndefined();
	});
});

describe('tierLayout', () => {
	it('gives the largest avatar size that fits', () => {
		const counts = [12, 30, 41, 25, 9];
		const { size, gap } = tierLayout(counts, 1776, 720);
		expect(height(counts, 1776, size, gap)).toBeLessThanOrEqual(720);
		const next = size + 1;
		expect(height(counts, 1776, next, Math.ceil(next / 8))).toBeGreaterThan(720);
	});

	it('keeps 1 line for empty rows', () => {
		expect(tierLayout([0, 0, 0, 0, 0], 900, 2000).size).toBe(160);
	});
});
