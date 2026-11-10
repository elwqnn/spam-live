import { describe, expect, it } from 'vitest';
import { squarify } from './treemap';

describe('squarify', () => {
	it('fills the box with areas in proportion and no overlap', () => {
		const values = [6, 6, 4, 3, 2, 2, 1];
		const box = { x: 10, y: 20, w: 600, h: 400 };
		const rects = squarify(values, (v) => v, box);
		const total = values.reduce((a, b) => a + b, 0);
		expect(rects).toHaveLength(values.length);
		for (const r of rects) {
			expect(r.w * r.h).toBeCloseTo((r.item / total) * box.w * box.h, 6);
			expect(r.x).toBeGreaterThanOrEqual(box.x - 1e-9);
			expect(r.y).toBeGreaterThanOrEqual(box.y - 1e-9);
			expect(r.x + r.w).toBeLessThanOrEqual(box.x + box.w + 1e-9);
			expect(r.y + r.h).toBeLessThanOrEqual(box.y + box.h + 1e-9);
		}
		for (const a of rects)
			for (const b of rects) {
				if (a === b) continue;
				const dx = Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x);
				const dy = Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y);
				expect(dx <= 1e-9 || dy <= 1e-9).toBe(true);
			}
	});

	it('returns nothing for an empty box or no value', () => {
		expect(squarify([1, 2], (v) => v, { x: 0, y: 0, w: 0, h: 10 })).toEqual([]);
		expect(squarify([0, 0], (v) => v, { x: 0, y: 0, w: 10, h: 10 })).toEqual([]);
	});
});
