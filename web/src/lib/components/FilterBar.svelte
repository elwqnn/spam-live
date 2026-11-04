<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';
	import FilterMenu from './FilterMenu.svelte';

	let {
		filters,
		actions,
		reset,
		label = m.filters()
	}: {
		filters: Filter[];
		actions?: Snippet;
		reset?: Record<string, null>;
		label?: string;
	} = $props();
</script>

<div role="group" aria-label={label} class="flex flex-wrap items-center gap-2">
	{#each filters as filter (filter.id)}
		<FilterMenu {filter} {reset} />
	{/each}
	{#if actions}
		<div class="ml-auto flex flex-wrap items-center gap-2">{@render actions()}</div>
	{/if}
</div>
