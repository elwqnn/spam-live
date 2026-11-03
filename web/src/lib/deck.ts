export const NAME_MAX = 60;
export const GUIDE_MAX = 50_000;

export type Slot = { id: string; talent: number } | null;
export type DeckInput = { name: string; cards: string[]; talents: number[]; guide: string };
export type Issue = 'cards' | 'talents' | 'name' | 'guide';

export const chars = (text: string) => [...text].length;

export const text = (form: FormData, key: string) =>
	String(form.get(key) ?? '').replace(/\r\n?/g, '\n');

export function pick(slots: Slot[], id: string, at?: number): Slot[] {
	if (slots.some((s) => s?.id === id)) return slots;
	const index = at ?? slots.indexOf(null);
	if (index < 0 || index >= slots.length) return slots;
	return slots.map((s, i) => (i === index ? { id, talent: 0 } : s));
}

export function issues(deck: DeckInput): Issue[] {
	const out: Issue[] = [];
	if (deck.cards.length !== 4 || new Set(deck.cards).size !== 4) out.push('cards');
	if (deck.talents.length !== 4 || deck.talents.some((t) => ![0, 1, 2].includes(t)))
		out.push('talents');
	const name = chars(deck.name.trim());
	if (name < 1 || name > NAME_MAX) out.push('name');
	if (chars(deck.guide) > GUIDE_MAX) out.push('guide');
	return out;
}

export function readDeck(form: FormData): DeckInput {
	const cards = form.getAll('card').map(String);
	return {
		name: text(form, 'name'),
		cards,
		talents: cards.map((id) => {
			const value = form.get(`talent-${id}`);
			return value === null ? -1 : Number(value);
		}),
		guide: text(form, 'guide')
	};
}
