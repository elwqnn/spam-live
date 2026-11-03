import type { Handle, HandleFetch } from '@sveltejs/kit/hooks';
import { API_URL } from '$app/env/private';
import { getTextDirection } from '#lib/paraglide/runtime.js';
import { paraglideMiddleware } from '#lib/paraglide/server.js';

export const handle: Handle = ({ event, resolve }) =>
	paraglideMiddleware(event.request, ({ request, locale }) => {
		return resolve(
			{ ...event, request },
			{
				filterSerializedResponseHeaders: (name) =>
					name === 'content-length' || name === 'transfer-encoding',
				transformPageChunk: ({ html }) =>
					html
						.replace('%paraglide.lang%', locale)
						.replace('%paraglide.dir%', getTextDirection(locale))
			}
		);
	});

export const handleFetch: HandleFetch = ({ event, request, fetch }) => {
	const prefix = `${event.url.origin}/api/`;
	if (!request.url.startsWith(prefix)) return fetch(request);
	const target = new Request(API_URL + request.url.slice(event.url.origin.length), request);
	const cookie = event.request.headers.get('cookie');
	if (cookie) target.headers.set('cookie', cookie);
	return fetch(target);
};
