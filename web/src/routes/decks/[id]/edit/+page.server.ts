import { redirect } from '@sveltejs/kit';
import { api, href, result } from '#lib';
import { readDeck } from '#lib/deck.js';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async ({ fetch, url, params, request }) => {
		const id = Number(params.id);
		const body = readDeck(await request.formData());
		const res = await api(fetch, url.origin).PATCH('/api/decks/{id}', {
			params: { path: { id } },
			body
		});
		if (!res.response.ok) return result('save', res);
		redirect(303, href(`/decks/${id}`));
	}
};
