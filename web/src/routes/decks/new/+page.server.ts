import { redirect } from '@sveltejs/kit';
import { api, href, result } from '#lib';
import { readDeck } from '#lib/deck.js';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async ({ fetch, url, request }) => {
		const body = readDeck(await request.formData());
		const res = await api(fetch, url.origin).POST('/api/decks', { body });
		if (!res.data) return result('save', res);
		redirect(303, href(`/decks/${res.data.id}`));
	}
};
