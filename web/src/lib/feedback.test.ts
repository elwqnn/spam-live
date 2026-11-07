import { describe, expect, it } from 'vitest';
import {
	blockSize,
	captureScale,
	clip,
	elementText,
	pagePath,
	pixelate,
	selector,
	siteVersion,
	validBody,
	type Picked
} from './feedback';

type Fake = Picked & { children: Fake[]; parentElement: Fake | null };

function el(tagName: string, attrs: { id?: string; cls?: string } = {}, kids: Fake[] = []): Fake {
	const node: Fake = {
		id: attrs.id ?? '',
		tagName: tagName.toUpperCase(),
		classList: (attrs.cls ?? '').split(' ').filter(Boolean),
		parentElement: null,
		children: kids
	};
	for (const kid of kids) kid.parentElement = node;
	return node;
}

const never = () => false;

describe('selector', () => {
	it('uses a unique id', () => {
		const target = el('button', { id: 'send' });
		el('body', {}, [el('main', {}, [target])]);
		expect(selector(target, (id) => id === 'send')).toBe('#send');
	});

	it('ignores an id that is not unique or not a plain name', () => {
		const target = el('button', { id: 'a:b' });
		el('body', {}, [target]);
		expect(selector(target, () => true)).toBe('button');
		const twice = el('span', { id: 'twice' });
		el('body', {}, [twice]);
		expect(selector(twice, never)).toBe('span');
	});

	it('keeps max 2 safe class names', () => {
		const target = el('a', {
			cls: 'sm:flex px-(--gutter) w-[3rem] gap-1.5 w-1/2 eyebrow text-xs hover:x'
		});
		el('body', {}, [el('nav', {}, [target])]);
		expect(selector(target, never)).toBe('nav > a.eyebrow.text-xs');
	});

	it('adds nth-of-type only when a sibling matches', () => {
		const a = el('li', { cls: 'row' });
		const b = el('li', { cls: 'row' });
		const c = el('li', { cls: 'other' });
		el('body', {}, [el('ul', {}, [a, b, c])]);
		expect(selector(b, never)).toBe('ul > li.row:nth-of-type(2)');
		expect(selector(c, never)).toBe('ul > li.other');
	});

	it('stops at an ancestor with a unique id', () => {
		const target = el('span');
		el('body', {}, [el('div', { id: 'deck' }, [el('p', {}, [target])])]);
		expect(selector(target, (id) => id === 'deck')).toBe('#deck > p > span');
	});

	it('goes max 4 levels up', () => {
		const target = el('i');
		el('body', {}, [
			el('a', {}, [el('b', {}, [el('c', {}, [el('d', {}, [el('e', {}, [target])])])])])
		]);
		expect(selector(target, never)).toBe('b > c > d > e > i');
	});

	it('cuts the selector to 300 characters', () => {
		const long = 'x'.repeat(120);
		const target = el('span', { cls: long });
		el('body', {}, [el('div', { cls: long }, [el('p', { cls: long }, [target])])]);
		const result = selector(target, never);
		expect(result.length).toBeLessThanOrEqual(300);
		expect(result.endsWith(`span.${long}`)).toBe(true);
		const huge = el('span', { cls: 'y'.repeat(400) });
		el('body', {}, [huge]);
		expect(selector(huge, never).length).toBe(300);
	});
});

function readable(text: string, privateParts: string[], inside = false) {
	const el = {
		closest: () => (inside ? {} : null),
		cloneNode: () => {
			const copy = {
				textContent: text as string | null,
				querySelectorAll: () =>
					privateParts.map((part) => ({
						remove: () => {
							copy.textContent = copy.textContent?.replace(part, '') ?? null;
						}
					}))
			};
			return copy;
		}
	};
	return el as unknown as Element;
}

describe('element text', () => {
	it('collapses whitespace and keeps 200 characters', () => {
		expect(clip('  a \n\t b  ')).toBe('a b');
		expect([...clip('é'.repeat(300))].length).toBe(200);
		expect(clip('🙂'.repeat(201))).toBe('🙂'.repeat(200));
	});

	it('removes private parts', () => {
		expect(elementText(readable('By  Alice on deck', ['Alice']))).toBe('By on deck');
	});

	it('sends no text inside a private element', () => {
		expect(elementText(readable('Alice', [], true))).toBe('');
	});
});

describe('feedback helpers', () => {
	it('checks the body length after trim', () => {
		expect(validBody('   short   ')).toBe(false);
		expect(validBody('0123456789')).toBe(true);
		expect(validBody('🙂'.repeat(10))).toBe(true);
		expect(validBody('a'.repeat(5000))).toBe(true);
		expect(validBody('a'.repeat(5001))).toBe(false);
	});

	it('reads the site version', () => {
		expect(siteVersion(undefined)).toBe('dev');
		expect(siteVersion('')).toBe('dev');
		expect(siteVersion('abc1234')).toBe('abc1234');
		expect(siteVersion('ABC1234')).toBe('dev');
		expect(siteVersion('abc12')).toBe('dev');
	});

	it('makes a safe page path', () => {
		expect(pagePath('/fr/cards', '?sort=name')).toBe('/fr/cards?sort=name');
		expect(pagePath('//evil', '')).toBe('/evil');
		expect(pagePath('/decks', '?q=a`b\\c"<>')).toBe('/decks?q=a%60b%5Cc%22%3C%3E');
		expect(pagePath('/decks', '?q=a b')).toBe('/decks?q=a%20b');
		expect(pagePath('/x', `?q=${'a'.repeat(3000)}`).length).toBe(2000);
	});

	it('limits the capture scale to 4096 px on the long side', () => {
		expect(captureScale(1280, 800, 2)).toBe(2);
		expect(captureScale(2560, 1440, 2)).toBe(1.6);
		expect(captureScale(375, 812, 0)).toBe(1);
		expect(blockSize(1)).toBe(12);
		expect(blockSize(3)).toBe(24);
	});

	it('pixelates with the mean of each block', () => {
		const data = new Uint8ClampedArray([
			0, 0, 0, 255, 100, 100, 100, 255, 50, 0, 0, 255, 0, 0, 0, 255, 200, 200, 200, 255, 50, 0, 0,
			255
		]);
		pixelate(data, 3, 2, 2);
		expect([...data.slice(0, 4)]).toEqual([75, 75, 75, 255]);
		expect([...data.slice(4, 8)]).toEqual([75, 75, 75, 255]);
		expect([...data.slice(12, 16)]).toEqual([75, 75, 75, 255]);
		expect([...data.slice(8, 12)]).toEqual([50, 0, 0, 255]);
	});
});
