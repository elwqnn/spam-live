import { onMount } from 'svelte';

export function hydrated() {
	let ready = $state(false);
	onMount(() => {
		ready = true;
	});
	return {
		lock: (busy = false) => ({
			disabled: busy || !ready,
			'data-wait': busy || ready ? undefined : ''
		})
	};
}
