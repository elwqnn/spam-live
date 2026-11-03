import { api, result } from '#lib';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async ({ fetch, url, request }) => {
		const userCode = String((await request.formData()).get('userCode') ?? '');
		return result(
			'link',
			await api(fetch, url.origin).POST('/api/device/confirm', { body: { userCode } })
		);
	}
};
