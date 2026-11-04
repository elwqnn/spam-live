<script lang="ts">
	import { page } from '$app/state';
	import type { Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';
	import FilterMenu from './FilterMenu.svelte';

	const all = $derived(page.url.searchParams.get('version') === 'all');
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

<FilterMenu {filter} selected={all ? null : undefined} />
