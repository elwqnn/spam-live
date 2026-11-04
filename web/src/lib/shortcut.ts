export const GO = {
	d: '/decks',
	c: '/cards',
	s: '/stats',
	i: '/import',
	l: '/glossary'
} as const;

export const SEQUENCE_MS = 1000;

export type Shortcut = 'palette' | 'search' | 'help' | 'g' | (typeof GO)[keyof typeof GO];

export type Press = {
	key: string;
	ctrlKey: boolean;
	altKey: boolean;
	metaKey: boolean;
	shiftKey: boolean;
	isComposing: boolean;
	timeStamp: number;
	target: unknown;
};

const EDITABLE = ['INPUT', 'TEXTAREA', 'SELECT'];

const editable = (target: unknown) => {
	const el = target as { tagName?: string; isContentEditable?: boolean } | null;
	return !!el && (EDITABLE.includes(el.tagName ?? '') || !!el.isContentEditable);
};

export const isMac = () => /Mac|iPhone|iPad|iPod/.test(navigator.platform);

export function shortcut(e: Press, g: number | null, mac: boolean): Shortcut | null {
	if (e.isComposing) return null;
	const [mod, other] = mac ? [e.metaKey, e.ctrlKey] : [e.ctrlKey, e.metaKey];
	if (mod && !other && !e.altKey && !e.shiftKey && e.key.toLowerCase() === 'k') return 'palette';
	if (e.ctrlKey || e.altKey || e.metaKey || editable(e.target)) return null;
	if (g !== null && e.timeStamp - g < SEQUENCE_MS && Object.hasOwn(GO, e.key))
		return GO[e.key as keyof typeof GO];
	if (e.key === 'g') return 'g';
	if (e.key === '/') return 'search';
	if (e.key === '?') return 'help';
	return null;
}
