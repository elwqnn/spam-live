import { fail } from '@sveltejs/kit';
import type { SubmitFunction } from '$app/forms';

export * from '@spam-live/ui';
export { apiBase, art, client, href, loginHref, queryHref, writes } from './host.js';

export const formAction = (url: { search: string }, name: string) =>
	`?/${name}${url.search.replace('?', '&')}`;

export function result(key: string, res: { error?: unknown; response: Response }) {
	if (res.response.ok) return { key, error: null };
	const body = res.error as { error?: string } | undefined;
	return fail(res.response.status, { key, error: body?.error ?? res.response.statusText });
}

export const closeAfter: SubmitFunction =
	({ submitter }) =>
	async ({ result, update, formElement }) => {
		(submitter?.closest('dialog') ?? formElement.closest('dialog'))?.close();
		if (result.type === 'success') formElement.closest('details')?.removeAttribute('open');
		await update();
	};
