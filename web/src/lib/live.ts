import { invalidate } from '$app/navigation';
import { SvelteMap } from 'svelte/reactivity';

export type Votes = { id: number; upvotes: number; downvotes: number };

export const votes = new SvelteMap<number, Votes>();

const listeners = new Set<(sync: boolean) => void>();
let users = 0;
let watching = false;
let source: EventSource | undefined;
let retry: ReturnType<typeof setTimeout> | undefined;
let reload: ReturnType<typeof setTimeout> | undefined;

function stale() {
	votes.clear();
	clearTimeout(reload);
	reload = setTimeout(() => invalidate('app:decks'), 500);
}

function emit(sync: boolean) {
	if (sync) stale();
	for (const listener of listeners) listener(sync);
}

function close() {
	clearTimeout(retry);
	clearTimeout(reload);
	source?.close();
	source = undefined;
	votes.clear();
}

function connect() {
	close();
	if (!users || document.visibilityState !== 'visible') return;
	let opened = false;
	const events = (source = new EventSource('/api/decks/events'));
	events.addEventListener('decks', (event) => {
		if (event.data === '0') stale();
		emit(false);
	});
	events.addEventListener('votes', (event) => {
		const counts: Votes = JSON.parse(event.data);
		votes.set(counts.id, counts);
	});
	events.addEventListener('open', () => {
		if (opened) emit(true);
		opened = true;
	});
	events.addEventListener('error', () => {
		if (events.readyState !== EventSource.CLOSED) return;
		clearTimeout(retry);
		retry = setTimeout(resync, 5000);
	});
}

function resync() {
	connect();
	if (source) emit(true);
}

export function live(listener?: (sync: boolean) => void) {
	if (listener) listeners.add(listener);
	if (!watching) document.addEventListener('visibilitychange', resync);
	watching = true;
	if (!users++ && !source) connect();
	return () => {
		if (listener) listeners.delete(listener);
		if (--users) return;
		setTimeout(() => {
			if (!users) close();
		});
	};
}
