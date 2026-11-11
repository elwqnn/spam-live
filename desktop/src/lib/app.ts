export const SITE: string = import.meta.env.VITE_SITE_URL;

export type Report = {
	kind: 'matches' | 'decks';
	at: number;
	counts: Record<string, number>;
	reasons: string[];
	error: string | null;
};
export type Notice =
	{ kind: 'unlinked' | 'expired' | 'revoked' } | { kind: 'error'; message: string };
export type Status = {
	version: string;
	linked: boolean;
	code: string | null;
	notice: Notice | null;
	folder: string | null;
	custom: boolean;
	found: boolean;
	last: Report | null;
	update: boolean;
	ready: string | null;
	manual: boolean;
	autostart: boolean;
};
export type UploaderState = 'linking' | 'ready' | 'update' | 'unlinked' | 'folder' | 'watching';
export type Move = 'back' | 'forward' | 'reload';
export type Target = { go: string } | { open: string } | null;

const ROUTES = [
	/^\/$/,
	/^\/(cards|decks|stats|glossary|me|credits|privacy|settings)$/,
	/^\/cards\/[^/]+$/,
	/^\/decks\/(new|\d+|\d+\/edit)$/
];
const LOCALE = /^\/(en|fr)(?=\/|$)/;

const base = (url: URL) => `${url.protocol}//${url.host}`;

export const appRoute = (pathname: string) => ROUTES.some((route) => route.test(pathname));

export const siteUrl = (path: string, locale: string) =>
	`${SITE}${locale === 'en' ? '' : `/${locale}`}${path}`;

export const pathname = (path: string) => new URL(path, 'http://app').pathname;

export function target(link: URL, app: URL, locale: string, blank = false): Target {
	const tail = link.search + link.hash;
	const web = /^https?:$/.test(link.protocol);
	if (base(link) === base(app)) {
		if (!appRoute(link.pathname)) return { open: siteUrl(link.pathname + tail, locale) };
		return blank ? { go: link.pathname + tail } : null;
	}
	if (base(link) === base(new URL(SITE)) && !blank) {
		const path = link.pathname.replace(LOCALE, '') || '/';
		return appRoute(path) ? { go: path + tail } : { open: link.href };
	}
	return web ? { open: link.href } : null;
}

export const account = (method: string, path: string) =>
	(method !== 'GET' && method !== 'HEAD') || /^\/api\/(me|decks)(\/|$)/.test(path);

export function uploaderState(status: Status | null): UploaderState | null {
	if (!status) return null;
	if (status.code) return 'linking';
	if (status.ready) return 'ready';
	if (status.update || status.manual) return 'update';
	if (!status.linked) return 'unlinked';
	return status.found ? 'watching' : 'folder';
}

export function keyMove(e: {
	key: string;
	altKey: boolean;
	ctrlKey: boolean;
	metaKey: boolean;
	shiftKey: boolean;
}): Move | null {
	if (e.metaKey) return null;
	if (e.altKey && !e.ctrlKey && !e.shiftKey)
		return e.key === 'ArrowLeft' ? 'back' : e.key === 'ArrowRight' ? 'forward' : null;
	if (e.altKey) return null;
	return e.key === 'F5' || (e.ctrlKey && e.key.toLowerCase() === 'r') ? 'reload' : null;
}

export const mouseMove = (button: number): Move | null =>
	button === 3 ? 'back' : button === 4 ? 'forward' : null;
