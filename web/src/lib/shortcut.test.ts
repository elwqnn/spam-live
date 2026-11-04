import { describe, expect, it } from 'vitest';
import { shortcut, type Press } from './shortcut';

const press = (key: string, more: Partial<Press> = {}): Press => ({
	key,
	ctrlKey: false,
	altKey: false,
	metaKey: false,
	shiftKey: false,
	isComposing: false,
	timeStamp: 0,
	target: { tagName: 'BODY', isContentEditable: false },
	...more
});

describe('shortcut', () => {
	it('opens the palette with Ctrl+K, or Cmd+K on macOS', () => {
		expect(shortcut(press('k', { ctrlKey: true }), null, false)).toBe('palette');
		expect(shortcut(press('K', { ctrlKey: true }), null, false)).toBe('palette');
		expect(shortcut(press('k', { metaKey: true }), null, true)).toBe('palette');
		expect(shortcut(press('k', { metaKey: true }), null, false)).toBeNull();
		expect(shortcut(press('k', { ctrlKey: true }), null, true)).toBeNull();
	});

	it('opens the palette from a text field', () => {
		const target = { tagName: 'INPUT' };
		expect(shortcut(press('k', { ctrlKey: true, target }), null, false)).toBe('palette');
	});

	it('keeps other modifier shortcuts for the browser', () => {
		expect(shortcut(press('K', { ctrlKey: true, shiftKey: true }), null, false)).toBeNull();
		expect(shortcut(press('k', { ctrlKey: true, altKey: true }), null, false)).toBeNull();
		expect(shortcut(press('/', { ctrlKey: true }), null, false)).toBeNull();
		expect(shortcut(press('g', { altKey: true }), null, false)).toBeNull();
		expect(shortcut(press('?', { metaKey: true }), null, true)).toBeNull();
		expect(shortcut(press('c', { ctrlKey: true }), 0, false)).toBeNull();
	});

	it('reads the single keys', () => {
		expect(shortcut(press('/'), null, false)).toBe('search');
		expect(shortcut(press('?', { shiftKey: true }), null, false)).toBe('help');
		expect(shortcut(press('g'), null, false)).toBe('g');
		expect(shortcut(press('x'), null, false)).toBeNull();
		expect(shortcut(press('c'), null, false)).toBeNull();
	});

	it('reads the g sequences', () => {
		const at = (key: string) => shortcut(press(key, { timeStamp: 900 }), 100, false);
		expect(at('d')).toBe('/decks');
		expect(at('c')).toBe('/cards');
		expect(at('s')).toBe('/stats');
		expect(at('i')).toBe('/import');
		expect(at('l')).toBe('/glossary');
		expect(at('g')).toBe('g');
		expect(at('x')).toBeNull();
		expect(at('toString')).toBeNull();
	});

	it('stops a sequence after 1 second', () => {
		expect(shortcut(press('c', { timeStamp: 1099 }), 100, false)).toBe('/cards');
		expect(shortcut(press('c', { timeStamp: 1100 }), 100, false)).toBeNull();
	});

	it('ignores keys in text fields and editable elements', () => {
		for (const target of [
			{ tagName: 'INPUT' },
			{ tagName: 'TEXTAREA' },
			{ tagName: 'SELECT' },
			{ tagName: 'DIV', isContentEditable: true }
		]) {
			expect(shortcut(press('g', { target }), null, false)).toBeNull();
			expect(shortcut(press('/', { target }), null, false)).toBeNull();
			expect(shortcut(press('c', { target, timeStamp: 10 }), 0, false)).toBeNull();
		}
		expect(shortcut(press('g', { target: null }), null, false)).toBe('g');
	});

	it('ignores keys during IME composition', () => {
		expect(shortcut(press('g', { isComposing: true }), null, false)).toBeNull();
		expect(shortcut(press('k', { ctrlKey: true, isComposing: true }), null, false)).toBeNull();
	});
});
