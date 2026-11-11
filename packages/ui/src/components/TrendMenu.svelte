<script lang="ts">
	import { pageUrl } from '#ui/modal.js';
	import type { Filter } from '#ui/filter.js';
	import { m } from '$paraglide/messages.js';
	import FilterChips from './FilterChips.svelte';
	import FilterMenu from './FilterMenu.svelte';

	let { flat = false }: { flat?: boolean } = $props();

	const all = $derived(pageUrl().searchParams.get('version') === 'all');
	const filter: Filter = $derived({
		id: 'trend',
		label: m.trend(),
		default: 'week',
		options: [
			{ value: 'week', label: m.trend_week() },
			...(all ? [] : [{ value: 'version', label: m.version() }])
		]
	});
</script>

{#if flat}
	<FilterChips {filter} selected={all ? null : undefined} />
{:else}
	<FilterMenu {filter} selected={all ? null : undefined} />
{/if}
