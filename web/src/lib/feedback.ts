export const KINDS = ['bug', 'idea', 'other'] as const;
export type Kind = (typeof KINDS)[number];

export const BODY_MIN = 10;
export const BODY_MAX = 5000;
export const MAX_BYTES = 3 * 1024 * 1024;
export const MAX_SIDE = 4096;
export const TEXT_MAX = 200;
export const SELECTOR_MAX = 300;

export const validBody = (text: string) => {
	const n = [...text.trim()].length;
	return n >= BODY_MIN && n <= BODY_MAX;
};

export const siteVersion = (value: string | undefined) =>
	value && /^[0-9a-f]{7,40}$/.test(value) ? value : 'dev';

export const pagePath = (pathname: string, search: string) =>
	(pathname + search)
		.replace(/[^\x21-\x7e]|[`<>"\\]/gu, (c) => encodeURIComponent(c))
		.replace(/^\/+/, '/')
		.slice(0, 2000);

export const captureScale = (width: number, height: number, ratio: number) =>
	Math.min(ratio || 1, MAX_SIDE / Math.max(width, height, 1));

export const blockSize = (scale: number) => Math.max(12, Math.round(8 * scale));

export function pixelate(data: Uint8ClampedArray, width: number, height: number, block: number) {
	for (let by = 0; by < height; by += block) {
		for (let bx = 0; bx < width; bx += block) {
			const xs = Math.min(bx + block, width);
			const ys = Math.min(by + block, height);
			const sum = [0, 0, 0, 0];
			for (let y = by; y < ys; y++)
				for (let x = bx; x < xs; x++)
					for (let c = 0; c < 4; c++) sum[c] += data[(y * width + x) * 4 + c];
			const n = (xs - bx) * (ys - by);
			for (let y = by; y < ys; y++)
				for (let x = bx; x < xs; x++)
					for (let c = 0; c < 4; c++) data[(y * width + x) * 4 + c] = Math.round(sum[c] / n);
		}
	}
}

export type Picked = {
	id: string;
	tagName: string;
	classList: Iterable<string>;
	parentElement: Picked | null;
	children: ArrayLike<Picked>;
};

const IDENT = /^-?[A-Za-z_][\w-]*$/;
const STOP = ['BODY', 'HTML'];

function part(el: Picked) {
	const tag = el.tagName.toLowerCase();
	const classes = [...el.classList].filter((c) => IDENT.test(c)).slice(0, 2);
	const same = (s: Picked) =>
		s.tagName === el.tagName && classes.every((c) => [...s.classList].includes(c));
	const siblings = el.parentElement ? Array.from(el.parentElement.children) : [el];
	const sameTag = siblings.filter((s) => s.tagName === el.tagName);
	const nth = siblings.filter((s) => s !== el && same(s)).length
		? `:nth-of-type(${sameTag.indexOf(el) + 1})`
		: '';
	return tag + classes.map((c) => `.${c}`).join('') + nth;
}

export function selector(el: Picked, unique: (id: string) => boolean) {
	const parts: string[] = [];
	for (
		let node: Picked | null = el;
		node && !STOP.includes(node.tagName) && parts.length < 5;
		node = node.parentElement
	) {
		if (IDENT.test(node.id) && unique(node.id)) {
			parts.unshift(`#${node.id}`);
			break;
		}
		parts.unshift(part(node));
	}
	while (parts.length > 1 && parts.join(' > ').length > SELECTOR_MAX) parts.shift();
	return (parts.join(' > ') || el.tagName.toLowerCase()).slice(0, SELECTOR_MAX);
}

export const clip = (text: string, max = TEXT_MAX) =>
	[...text.replace(/\s+/g, ' ').trim()].slice(0, max).join('');

export function elementText(el: Element) {
	if (el.closest('[data-private]')) return '';
	const copy = el.cloneNode(true) as Element;
	for (const node of copy.querySelectorAll('[data-private]')) node.remove();
	return clip(copy.textContent ?? '');
}
