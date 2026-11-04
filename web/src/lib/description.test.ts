import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { formatDescription, matchKeyword } from './description';

type Raw = { id: string; description: Record<string, string | null> | null };
type Keyword = { id: string; name: Record<string, string> };
const catalog: Record<'abilities' | 'talents' | 'tox', Raw[]> & { keywords: Keyword[] } =
	JSON.parse(readFileSync(new URL('../../../catalog/1.28.1.json', import.meta.url), 'utf8'));

function fmt(kind: 'abilities' | 'talents' | 'tox', id: string, lang: 'en' | 'fr') {
	const entry = catalog[kind].find((e) => e.id === id);
	const text = entry?.description?.[lang];
	if (!entry || !text) throw new Error(`no ${lang} text for ${id}`);
	return formatDescription(text, { data: entry });
}

describe('formatDescription', () => {
	it('renders element tokens and dynamic values', () => {
		expect(fmt('abilities', 'Brulure nocturne', 'en')).toEqual([
			{ type: 'text', text: 'Deal ' },
			{ type: 'value', text: '25', dynamic: true },
			{ type: 'text', text: ' ' },
			{ type: 'element', element: 'fire', label: 'FIRE' },
			{ type: 'text', text: ' and ' },
			{ type: 'value', text: '25', dynamic: true },
			{ type: 'text', text: ' ' },
			{ type: 'element', element: 'shadow', label: 'SHADOW' },
			{ type: 'text', text: ' damage to all enemy cards' }
		]);
	});

	it('turns newlines into line breaks', () => {
		const out = fmt('abilities', 'Brulure nocturne', 'fr');
		expect(out.slice(0, 3)).toEqual([
			{ type: 'text', text: 'Cible toutes les cartes adverses' },
			{ type: 'br' },
			{ type: 'text', text: 'Attaque ' }
		]);
		expect(out).toContainEqual({ type: 'element', element: 'fire', label: 'FEU' });
	});

	it('keeps an accented label', () => {
		expect(fmt('abilities', 'Ecrit luminescent', 'fr').slice(0, 2)).toEqual([
			{ type: 'text', text: 'Bonus ' },
			{ type: 'element', element: 'light', label: 'LUMIÈRE' }
		]);
	});

	it('renders a keyword and a static value', () => {
		expect(fmt('abilities', 'Couvee protectrice', 'en')).toEqual([
			{ type: 'text', text: 'Apply ' },
			{ type: 'keyword', text: 'Taunt' },
			{ type: 'text', text: ' to a random card' },
			{ type: 'br' },
			{ type: 'text', text: 'Echo (' },
			{ type: 'value', text: '50', dynamic: false },
			{ type: 'text', text: '%)' }
		]);
	});

	it('keeps keywords with spaces and accents', () => {
		const ids = catalog.abilities
			.filter((e) => e.description?.fr?.includes('[Ennemis aléatoires]'))
			.map((e) => e.id);
		expect(ids.length).toBeGreaterThan(0);
		expect(fmt('abilities', ids[0], 'fr')).toContainEqual({
			type: 'keyword',
			text: 'Ennemis aléatoires'
		});
	});

	it('fills a min to max range and keeps the brackets', () => {
		expect(fmt('abilities', 'Vitesse folle', 'en').slice(0, 6)).toEqual([
			{ type: 'text', text: 'Deal [' },
			{ type: 'value', text: '9', dynamic: true },
			{ type: 'text', text: ' to ' },
			{ type: 'value', text: '10', dynamic: true },
			{ type: 'text', text: '] ' },
			{ type: 'element', element: 'nature', label: 'NATURE' }
		]);
		expect(fmt('abilities', 'Vitesse folle', 'fr')).toContainEqual({
			type: 'text',
			text: ' à '
		});
	});

	it('fills element tokens inside a cost bracket', () => {
		const en = fmt('abilities', 'Claque sombrevol', 'en');
		expect(en.slice(0, 3)).toEqual([
			{ type: 'text', text: '[Cost 2 ' },
			{ type: 'element', element: 'fire', label: 'FIRE' },
			{ type: 'text', text: ' & 2 ' }
		]);
		expect(en.slice(5, 8)).toEqual([
			{ type: 'element', element: 'nature', label: 'NATURE' },
			{ type: 'text', text: ']' },
			{ type: 'br' }
		]);
		expect(fmt('abilities', 'Claque sombrevol', 'fr').slice(0, 2)).toEqual([
			{ type: 'text', text: '[Coût 2 ' },
			{ type: 'element', element: 'fire', label: 'FEU' }
		]);
	});

	it('fills limited use values', () => {
		const tail = [
			{ type: 'value', text: '1', dynamic: true },
			{ type: 'text', text: '/' },
			{ type: 'value', text: '1', dynamic: false },
			{ type: 'text', text: ']' }
		];
		const en = fmt('abilities', 'Valse des nuages', 'en');
		expect(en.at(-5)).toEqual({ type: 'text', text: '[Limited Use ' });
		expect(en.slice(-4)).toEqual(tail);
		const fr = fmt('abilities', 'Valse des nuages', 'fr');
		expect(fr.at(-5)).toEqual({ type: 'text', text: '[Usage Limité ' });
		expect(fr.slice(-4)).toEqual(tail);
	});

	it('uses the mean of x and y for values', () => {
		expect(
			formatDescription('<value1> <min1> <max1>', {
				data: { actions: [{ value: { x: 2, y: 5 } }] }
			})
		).toEqual([
			{ type: 'value', text: '3', dynamic: false },
			{ type: 'text', text: ' ' },
			{ type: 'value', text: '2', dynamic: false },
			{ type: 'text', text: ' ' },
			{ type: 'value', text: '5', dynamic: false }
		]);
	});

	it('uses valueOfEffect for Tox', () => {
		const tox = catalog.tox.find((e) =>
			e.description?.en?.startsWith('This card increases its max HP')
		);
		expect(tox).toBeDefined();
		expect(formatDescription(tox!.description!.en!, { data: tox! })).toContainEqual({
			type: 'value',
			text: '60',
			dynamic: false
		});
	});

	it('shows the name of a missing value', () => {
		expect(fmt('abilities', 'Foudre', 'en')).toContainEqual({
			type: 'missing',
			name: 'missingHpBonusD'
		});
		expect(fmt('abilities', 'Ecrit sombre', 'fr')).toContainEqual({
			type: 'missing',
			name: 'value1A'
		});
		expect(formatDescription('<value3>', { data: { actions: [] } })).toEqual([
			{ type: 'missing', name: 'value3' }
		]);
		expect(formatDescription('<duration>', { data: {} })).toEqual([
			{ type: 'missing', name: 'duration' }
		]);
	});

	it('formats all catalog texts without undefined', () => {
		for (const kind of ['abilities', 'talents', 'tox'] as const) {
			for (const entry of catalog[kind]) {
				for (const lang of ['en', 'fr']) {
					const text = entry.description?.[lang];
					if (!text) continue;
					const out = JSON.stringify(formatDescription(text, { data: entry }));
					expect(out).not.toContain('undefined');
					expect(out).not.toMatch(/<\w+>/);
				}
			}
		}
	});
});

describe('matchKeyword', () => {
	const match = (text: string, lang: 'en' | 'fr') =>
		matchKeyword(
			text,
			catalog.keywords.map((k) => ({ id: k.id, name: k.name[lang] }))
		)?.id ?? null;

	it('matches en keywords by name, then by the longest prefix', () => {
		const cases = {
			Taunt: 'provocation',
			Freeze: 'freeze',
			Eternity: 'eternity',
			Random: 'random',
			Choice: 'choice',
			'Random enemies': 'randomEnemies',
			Temporality: 'temporality',
			Vampirism: 'vampirism',
			Vanish: 'vanish',
			Smoked: 'smoked',
			'AOE II': 'aoe2',
			'Cost 100 HP': 'cost',
			'Random enemy': 'random',
			CHANCE: null,
			Immunity: null,
			Mystery: null
		};
		for (const [text, id] of Object.entries(cases)) expect(match(text, 'en'), text).toBe(id);
	});

	it('matches fr keywords on folded text', () => {
		expect(match('Coût 100 PV', 'fr')).toBe('cost');
		expect(match('Ennemis Aléatoires', 'fr')).toBe('randomEnemies');
		expect(match('Ennemi aléatoire', 'fr')).toBe(null);
	});

	it('needs a space after a prefix', () => {
		expect(match('Randomly', 'en')).toBe(null);
	});
});
