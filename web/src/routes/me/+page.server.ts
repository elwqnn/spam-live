import { fail, redirect } from '@sveltejs/kit';
import { api, href, result } from '#lib';
import { m } from '#lib/paraglide/messages.js';
import type { Actions } from './$types';

export const actions: Actions = {
	revoke: async ({ fetch, url, request }) => {
		const id = Number((await request.formData()).get('id'));
		const res = await api(fetch, url.origin).DELETE('/api/me/devices/{id}', {
			params: { path: { id } }
		});
		return result(`device-${id}`, res);
	},
	deleteAccount: async ({ fetch, url, request, cookies }) => {
		const word = m.delete_account_word();
		if (String((await request.formData()).get('confirm') ?? '').trim() !== word)
			return fail(400, { key: 'delete', error: m.delete_account_confirm({ word }) });
		const res = await api(fetch, url.origin).DELETE('/api/me');
		if (!res.response.ok) return result('delete', res);
		cookies.delete('session', { path: '/' });
		redirect(303, href('/'));
	},
	logout: async ({ fetch, url, cookies }) => {
		await api(fetch, url.origin).POST('/api/auth/logout');
		cookies.delete('session', { path: '/' });
		redirect(303, href('/'));
	}
};
