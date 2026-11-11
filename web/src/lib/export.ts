import { m } from './paraglide/messages.js';
import { getLocale } from './paraglide/runtime.js';

export const SIZES = [
	{ id: 'wide', width: 1920, height: 1080 },
	{ id: 'vertical', width: 1080, height: 1920 },
	{ id: 'square', width: 1080, height: 1080 }
] as const;

export type Size = (typeof SIZES)[number];

export const BADGE = 96;

const clean = (text: string) =>
	text
		.normalize('NFKD')
		.replace(/\p{M}/gu, '')
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-|-$/g, '');

export const slug = (...texts: string[]) => texts.map(clean).find(Boolean) ?? 'image';

export const fileName = (kind: string, slug: string, size: Size) =>
	`spam-live-${kind}-${slug}-${size.width}x${size.height}.png`;

export function credit(names: (string | null | undefined)[]) {
	const unique = [...new Set(names.filter((name): name is string => !!name))];
	if (unique.length > 1)
		return m.illustrations({ names: new Intl.ListFormat(getLocale()).format(unique) });
	return unique.length ? m.illustration({ name: unique[0] }) : undefined;
}

export function tierLayout(counts: number[], width: number, height: number) {
	for (let size = 160; size > 16; size--) {
		const gap = Math.ceil(size / 8);
		const line = Math.max(1, Math.floor((width - BADGE) / (size + gap)));
		const total = counts.reduce((sum, n) => {
			const lines = Math.max(1, Math.ceil(n / line));
			return sum + Math.max(BADGE, lines * (size + gap) - gap) + 2 * gap + 1;
		}, 0);
		if (total <= height) return { size, gap };
	}
	return { size: 16, gap: 2 };
}
