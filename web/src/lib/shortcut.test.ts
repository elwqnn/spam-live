import { describe, expect, it } from 'vitest';
import { shortcut, type Press } from './shortcut';

const press = (key: string, more: Partial<Press> = {}): Press => ({
	key,
	ctrlKey: false,
	altKey: false,
	metaKey: false,
	shiftKey: false,
	isComposing: false,
	target: { tagName: 'BODY', isContentEditable: false },
	...more
});

describe('shortcut', () => {
	it('opens the palette with Ctrl+K, or Cmd+K on macOS', () => {
		expect(shortcut(press('k', { ctrlKey: true }), false, true)).toBe('palette');
		expect(shortcut(press('K', { ctrlKey: true }), false, true)).toBe('palette');
		expect(shortcut(press('k', { metaKey: true }), true, true)).toBe('palette');
		expect(shortcut(press('k', { metaKey: true }), false, true)).toBeNull();
		expect(shortcut(press('k', { ctrlKey: true }), true, true)).toBeNull();
	});

	it('opens the palette from a text field', () => {
		const target = { tagName: 'INPUT' };
		expect(shortcut(press('k', { ctrlKey: true, target }), false, true)).toBe('palette');
	});

	it('keeps other modifier shortcuts for the browser', () => {
		expect(shortcut(press('K', { ctrlKey: true, shiftKey: true }), false, true)).toBeNull();
		expect(shortcut(press('k', { ctrlKey: true, altKey: true }), false, true)).toBeNull();
		expect(shortcut(press('/', { ctrlKey: true }), false, true)).toBeNull();
		expect(shortcut(press('g', { altKey: true }), false, true)).toBeNull();
		expect(shortcut(press('d', { ctrlKey: true }), false, true)).toBeNull();
		expect(shortcut(press('?', { metaKey: true }), true, true)).toBeNull();
	});

	it('reads the single keys', () => {
		expect(shortcut(press('/'), false, true)).toBe('search');
		expect(shortcut(press('?', { shiftKey: true }), false, true)).toBe('help');
		expect(shortcut(press('d'), false, true)).toBe('download');
		expect(shortcut(press('g'), false, true)).toBeNull();
		expect(shortcut(press('x'), false, true)).toBeNull();
		expect(shortcut(press('c'), false, true)).toBeNull();
		expect(shortcut(press('toString'), false, true)).toBeNull();
	});

	it('has no g key and no g sequence', () => {
		expect(shortcut(press('g'), false, true)).toBeNull();
		expect(shortcut(press('G', { shiftKey: true }), false, true)).toBeNull();
		expect(shortcut(press('c'), false, true)).toBeNull();
		expect(shortcut(press('s'), false, true)).toBeNull();
		expect(shortcut(press('i'), false, true)).toBeNull();
		expect(shortcut(press('l'), false, true)).toBeNull();
	});

	it('keeps only Ctrl+K when single keys are off', () => {
		for (const key of ['/', '?', 'd']) expect(shortcut(press(key), false, false)).toBeNull();
		expect(shortcut(press('k', { ctrlKey: true }), false, false)).toBe('palette');
		expect(shortcut(press('k', { metaKey: true }), true, false)).toBe('palette');
	});

	it('ignores keys in text fields and editable elements', () => {
		for (const target of [
			{ tagName: 'INPUT' },
			{ tagName: 'TEXTAREA' },
			{ tagName: 'SELECT' },
			{ tagName: 'DIV', isContentEditable: true }
		]) {
			for (const key of ['/', '?', 'd'])
				expect(shortcut(press(key, { target }), false, true)).toBeNull();
		}
		expect(shortcut(press('d', { target: null }), false, true)).toBe('download');
	});

	it('ignores keys during IME composition', () => {
		expect(shortcut(press('d', { isComposing: true }), false, true)).toBeNull();
		expect(shortcut(press('k', { ctrlKey: true, isComposing: true }), false, true)).toBeNull();
	});
});
