import { describe, expect, it } from 'vitest';
import { chars, issues, pick, readDeck, text, type Slot } from './deck';

const empty: Slot[] = [null, null, null, null];
const deck = { name: 'Deck', cards: ['a', 'b', 'c', 'd'], talents: [0, 1, 2, 0], guide: '' };

describe('pick', () => {
	it('fills the first empty slot with talent 0', () => {
		const slots = pick(pick(empty, 'a'), 'b');
		expect(slots).toEqual([{ id: 'a', talent: 0 }, { id: 'b', talent: 0 }, null, null]);
	});

	it('ignores a card that is already picked', () => {
		const slots = pick(empty, 'a');
		expect(pick(slots, 'a')).toBe(slots);
		expect(pick(slots, 'a', 2)).toBe(slots);
	});

	it('ignores a fifth card', () => {
		const full = ['a', 'b', 'c', 'd'].reduce((s, id) => pick(s, id), empty);
		expect(pick(full, 'e')).toBe(full);
	});

	it('replaces the given slot', () => {
		const full = ['a', 'b', 'c', 'd'].map((id) => ({ id, talent: 2 }));
		expect(pick(full, 'e', 1)).toEqual([
			{ id: 'a', talent: 2 },
			{ id: 'e', talent: 0 },
			{ id: 'c', talent: 2 },
			{ id: 'd', talent: 2 }
		]);
	});
});

describe('issues', () => {
	it('accepts a valid deck', () => {
		expect(issues(deck)).toEqual([]);
	});

	it('needs 4 distinct cards', () => {
		expect(issues({ ...deck, cards: ['a', 'b', 'c'] })).toContain('cards');
		expect(issues({ ...deck, cards: ['a', 'a', 'b', 'c'] })).toContain('cards');
	});

	it('needs 4 talent indexes from 0 to 2', () => {
		expect(issues({ ...deck, talents: [0, 1, 3, 0] })).toEqual(['talents']);
		expect(issues({ ...deck, talents: [0, -1, 2, 0] })).toEqual(['talents']);
		expect(issues({ ...deck, talents: [0, 1, 2] })).toEqual(['talents']);
		expect(issues({ ...deck, talents: [0, 0.5, 2, 0] })).toEqual(['talents']);
	});

	it('checks the name and guide lengths in characters', () => {
		expect(issues({ ...deck, name: '   ' })).toEqual(['name']);
		expect(issues({ ...deck, name: 'é'.repeat(60) })).toEqual([]);
		expect(issues({ ...deck, name: 'x'.repeat(61) })).toEqual(['name']);
		expect(issues({ ...deck, guide: 'g'.repeat(50_000) })).toEqual([]);
		expect(issues({ ...deck, guide: 'g'.repeat(50_001) })).toEqual(['guide']);
	});

	it('counts emoji as 1 character, like the API', () => {
		expect(issues({ ...deck, name: '😀'.repeat(60) })).toEqual([]);
		expect(issues({ ...deck, name: '😀'.repeat(61) })).toEqual(['name']);
		expect(issues({ ...deck, guide: '😀'.repeat(50_000) })).toEqual([]);
		expect(issues({ ...deck, guide: '😀'.repeat(50_001) })).toEqual(['guide']);
	});
});

describe('form data', () => {
	it('counts code points', () => {
		expect(chars('a😀')).toBe(2);
		expect(chars('😀'.repeat(3))).toBe(3);
	});

	it('reads the deck fields in card order and normalizes line breaks', () => {
		const form = new FormData();
		form.set('name', 'Deck');
		form.set('guide', 'a\r\nb\rc');
		for (const [id, talent] of [
			['b', '2'],
			['a', '1']
		]) {
			form.append('card', id);
			form.set(`talent-${id}`, talent);
		}
		form.append('card', 'c');
		expect(readDeck(form)).toEqual({
			name: 'Deck',
			cards: ['b', 'a', 'c'],
			talents: [2, 1, -1],
			guide: 'a\nb\nc'
		});
		expect(text(form, 'missing')).toBe('');
	});
});
