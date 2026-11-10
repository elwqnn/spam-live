import { invalidateAll } from '$app/navigation';
import type { Client, Schemas } from './index.js';
import type { DeckInput } from './deck.js';

export type Writes = {
	vote: (deck: number, value: 1 | -1 | 0) => Promise<void>;
	comment: (deck: number, body: string) => Promise<void>;
	editComment: (id: number, body: string) => Promise<void>;
	deleteComment: (id: number) => Promise<void>;
	report: (targetType: 'deck' | 'comment', targetId: number, reason: string) => Promise<void>;
	publish: (deck: number) => Promise<void>;
	unpublish: (deck: number) => Promise<void>;
	deleteDeck: (deck: number) => Promise<void>;
	createDeck: (deck: DeckInput) => Promise<number>;
	updateDeck: (deck: number, input: DeckInput) => Promise<void>;
	revokeDevice: (id: number) => Promise<void>;
	deleteAccount: () => Promise<void>;
	logout: () => Promise<void>;
	feedback: (form: FormData) => Promise<Response>;
};

export const message = (e: unknown) => (e instanceof Error ? e.message : String(e));

export function submit(
	run: (data: FormData) => Promise<unknown>,
	done: (error: string | null) => void,
	next: () => Promise<unknown> = invalidateAll
) {
	return async (e: SubmitEvent & { currentTarget: HTMLFormElement }) => {
		const by = e.submitter as HTMLButtonElement | null;
		if ((by?.hasAttribute('formmethod') ? by.formMethod : e.currentTarget.method) === 'dialog')
			return;
		e.preventDefault();
		const form = e.currentTarget;
		let error: string | null = null;
		try {
			await run(new FormData(form));
		} catch (err) {
			error = message(err);
		}
		(by?.closest('dialog') ?? form.closest('dialog'))?.close();
		done(error);
		if (error) return;
		form.closest('details')?.removeAttribute('open');
		form.reset();
		await next();
	};
}

function check(res: { error?: unknown; response: Response }) {
	if (res.response.ok) return;
	const body = res.error as { error?: string } | undefined;
	throw new Error(body?.error ?? res.response.statusText);
}

export function apiWrites(client: () => Client): Writes {
	const deck = (id: number) => ({ params: { path: { id } } });
	return {
		vote: async (id, value) =>
			check(
				value === 0
					? await client().DELETE('/api/decks/{id}/vote', deck(id))
					: await client().PUT('/api/decks/{id}/vote', { ...deck(id), body: { value } })
			),
		comment: async (id, body) =>
			check(await client().POST('/api/decks/{id}/comments', { ...deck(id), body: { body } })),
		editComment: async (id, body) =>
			check(await client().PATCH('/api/comments/{id}', { ...deck(id), body: { body } })),
		deleteComment: async (id) => check(await client().DELETE('/api/comments/{id}', deck(id))),
		report: async (targetType, targetId, reason) =>
			check(await client().POST('/api/reports', { body: { targetType, targetId, reason } })),
		publish: async (id) => check(await client().POST('/api/decks/{id}/publish', deck(id))),
		unpublish: async (id) => check(await client().POST('/api/decks/{id}/unpublish', deck(id))),
		deleteDeck: async (id) => check(await client().DELETE('/api/decks/{id}', deck(id))),
		createDeck: async (input) => {
			const res = await client().POST('/api/decks', { body: input });
			check(res);
			return res.data?.id ?? 0;
		},
		updateDeck: async (id, input) =>
			check(await client().PATCH('/api/decks/{id}', { ...deck(id), body: input })),
		revokeDevice: async (id) => check(await client().DELETE('/api/me/devices/{id}', deck(id))),
		deleteAccount: async () => check(await client().DELETE('/api/me')),
		logout: async () => check(await client().POST('/api/auth/logout')),
		feedback: async (form) =>
			(
				await client().POST('/api/feedback', {
					body: form as unknown as Schemas['FeedbackForm']
				})
			).response
	};
}
