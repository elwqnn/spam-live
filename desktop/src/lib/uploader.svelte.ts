import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { invalidateAll } from '$app/navigation';
import type { Status } from './app.js';

export const uploader = $state<{ status: Status | null }>({ status: null });

let started = false;

function set(next: Status) {
	const linked = uploader.status?.linked ?? false;
	uploader.status = next;
	if (linked !== next.linked) invalidateAll();
}

export function watch() {
	if (started || !isTauri()) return;
	started = true;
	listen<Status>('status', (event) => set(event.payload));
	invoke<Status>('get_status').then((status) => {
		if (!uploader.status) set(status);
	});
}
