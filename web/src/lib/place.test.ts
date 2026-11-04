import { describe, expect, it } from 'vitest';
import { place } from './place';

const size = { width: 288, height: 200 };
const view = { width: 1000, height: 800 };

describe('place', () => {
	it('puts the popup right of and below the pointer', () => {
		expect(place({ x: 100, y: 100 }, size, view)).toEqual({ left: 116, top: 116 });
	});

	it('flips to the left side at the right edge', () => {
		expect(place({ x: 800, y: 100 }, size, view)).toEqual({ left: 496, top: 116 });
	});

	it('flips above the pointer at the bottom edge', () => {
		expect(place({ x: 100, y: 700 }, size, view)).toEqual({ left: 116, top: 484 });
	});

	it('keeps an 8 px margin when no side fits', () => {
		expect(place({ x: 200, y: 150 }, size, { width: 400, height: 300 })).toEqual({
			left: 8,
			top: 8
		});
		expect(
			place({ x: 10, y: 10 }, { width: 500, height: 400 }, { width: 400, height: 300 })
		).toEqual({ left: 8, top: 8 });
	});

	it('puts the popup right of the element rect, top aligned', () => {
		const rect = { left: 100, top: 300, right: 160, bottom: 380 };
		expect(place(rect, size, view)).toEqual({ left: 168, top: 300 });
	});

	it('flips to the left of the element rect at the right edge', () => {
		const rect = { left: 800, top: 300, right: 860, bottom: 380 };
		expect(place(rect, size, view)).toEqual({ left: 504, top: 300 });
	});

	it('clamps the element rect top into the viewport', () => {
		expect(place({ left: 100, top: 700, right: 160, bottom: 780 }, size, view).top).toBe(592);
		expect(place({ left: 100, top: -50, right: 160, bottom: 30 }, size, view).top).toBe(8);
	});
});
