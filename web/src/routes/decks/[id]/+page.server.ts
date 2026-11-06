import { fail, redirect, type RequestEvent } from '@sveltejs/kit';
import { api, href, result } from '#lib';
import { text } from '#lib/deck.js';
import type { Actions } from './$types';

async function input(event: RequestEvent) {
	const id = Number(event.params.id);
	return {
		client: api(event.fetch, event.url.origin),
		deck: { params: { path: { id } } },
		form: await event.request.formData()
	};
}

const commentPath = (form: FormData) => ({ path: { id: Number(form.get('id')) } });

export const actions: Actions = {
	vote: async (event) => {
		const { client, deck, form } = await input(event);
		const vote = form.get('vote');
		if (vote === '0') return result('vote', await client.DELETE('/api/decks/{id}/vote', deck));
		if (vote !== '1' && vote !== '-1') {
			return fail(400, { key: 'vote', error: 'vote must be 1, -1 or 0' });
		}
		const body = { value: Number(vote) };
		return result('vote', await client.PUT('/api/decks/{id}/vote', { ...deck, body }));
	},
	comment: async (event) => {
		const { client, deck, form } = await input(event);
		const body = { body: text(form, 'body') };
		return result('comment', await client.POST('/api/decks/{id}/comments', { ...deck, body }));
	},
	editComment: async (event) => {
		const { client, form } = await input(event);
		const params = commentPath(form);
		const body = { body: text(form, 'body') };
		return result(
			`comment-${params.path.id}`,
			await client.PATCH('/api/comments/{id}', { params, body })
		);
	},
	deleteComment: async (event) => {
		const { client, form } = await input(event);
		const params = commentPath(form);
		return result(
			`comment-${params.path.id}`,
			await client.DELETE('/api/comments/{id}', { params })
		);
	},
	report: async (event) => {
		const { client, form } = await input(event);
		const targetType = form.get('targetType') === 'comment' ? 'comment' : 'deck';
		const targetId = Number(form.get('targetId'));
		const body = { targetType, targetId, reason: text(form, 'reason') } as const;
		return result(`report-${targetType}-${targetId}`, await client.POST('/api/reports', { body }));
	},
	publish: async (event) => {
		const { client, deck } = await input(event);
		return result('status', await client.POST('/api/decks/{id}/publish', deck));
	},
	unpublish: async (event) => {
		const { client, deck } = await input(event);
		return result('status', await client.POST('/api/decks/{id}/unpublish', deck));
	},
	delete: async (event) => {
		const { client, deck } = await input(event);
		const res = await client.DELETE('/api/decks/{id}', deck);
		if (!res.response.ok) return result('delete', res);
		redirect(303, `${href('/decks')}?tab=mine`);
	}
};
