import type { RequestEvent } from '@sveltejs/kit';
import { api, result } from '#lib';
import type { Actions } from './$types';

async function user(event: RequestEvent, action: 'ban' | 'unban' | 'unlink') {
	const id = Number((await event.request.formData()).get('userId'));
	const res = await api(event.fetch, event.url.origin).POST(`/api/admin/users/{id}/${action}`, {
		params: { path: { id } }
	});
	return result(action, res);
}

export const actions: Actions = {
	resolve: async ({ fetch, url, request }) => {
		const form = await request.formData();
		const id = Number(form.get('id'));
		const action = form.get('action') === 'delete' ? 'delete' : 'dismiss';
		const res = await api(fetch, url.origin).POST('/api/admin/reports/{id}/resolve', {
			params: { path: { id } },
			body: { action }
		});
		return result('resolve', res);
	},
	deleteImage: async ({ fetch, url, request }) => {
		const id = Number((await request.formData()).get('id'));
		const res = await api(fetch, url.origin).DELETE('/api/admin/feedback/{id}/image', {
			params: { path: { id } }
		});
		return result('deleteImage', res);
	},
	ban: (event) => user(event, 'ban'),
	unban: (event) => user(event, 'unban'),
	unlink: (event) => user(event, 'unlink')
};
