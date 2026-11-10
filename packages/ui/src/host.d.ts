declare module '$host' {
	import type { Client, Schemas } from '#ui';

	type Location = { pathname: string; search: string };

	export type PageData = {
		me: Schemas['Me'] | null;
		cardStats?: Schemas['CardStats'][];
	};

	export const href: (path: string) => string;
	export const queryHref: (url: Location, changes: Record<string, string | null>) => string;
	export const loginHref: (url: Location) => string;
	export const client: () => Client;
	export const apiBase: string;
	export const art: (path: string) => string;
}
