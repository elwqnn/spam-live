import { describe, expect, it, vi } from 'vitest';
import { modalKind } from './modal';

vi.mock('$app/state', () => ({ page: {} }));

describe('modalKind', () => {
	it('finds a card page', () => {
		expect(modalKind('/cards/abc')).toBe('card');
		expect(modalKind('/cards/abc/')).toBe('card');
		expect(modalKind('/fr/cards/abc')).toBe('card');
	});

	it('finds a deck page with a numeric id', () => {
		expect(modalKind('/decks/12')).toBe('deck');
		expect(modalKind('/decks/12/')).toBe('deck');
		expect(modalKind('/fr/decks/12')).toBe('deck');
	});

	it('refuses the other pages', () => {
		for (const path of [
			'/',
			'/cards',
			'/cards/',
			'/decks',
			'/decks/',
			'/decks/new',
			'/decks/1/edit',
			'/decks/1x',
			'/stats',
			'/fr',
			'/fr/decks',
			'/xx/cards/abc',
			'/cards/abc/more'
		])
			expect(modalKind(path), path).toBeNull();
	});
});
