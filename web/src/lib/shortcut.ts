const KEYS = new Map<string, Shortcut>([
	['/', 'search'],
	['?', 'help'],
	['d', 'download']
]);

export type Shortcut = 'palette' | 'search' | 'help' | 'download';

export type Press = {
	key: string;
	ctrlKey: boolean;
	altKey: boolean;
	metaKey: boolean;
	shiftKey: boolean;
	isComposing: boolean;
	target: unknown;
};

const EDITABLE = ['INPUT', 'TEXTAREA', 'SELECT'];

const editable = (target: unknown) => {
	const el = target as { tagName?: string; isContentEditable?: boolean } | null;
	return !!el && (EDITABLE.includes(el.tagName ?? '') || !!el.isContentEditable);
};

export const isMac = () => /Mac|iPhone|iPad|iPod/.test(navigator.platform);

export function shortcut(e: Press, mac: boolean, single: boolean): Shortcut | null {
	if (e.isComposing) return null;
	const [mod, other] = mac ? [e.metaKey, e.ctrlKey] : [e.ctrlKey, e.metaKey];
	if (mod && !other && !e.altKey && !e.shiftKey && e.key.toLowerCase() === 'k') return 'palette';
	if (!single || e.ctrlKey || e.altKey || e.metaKey || editable(e.target)) return null;
	return KEYS.get(e.key) ?? null;
}
