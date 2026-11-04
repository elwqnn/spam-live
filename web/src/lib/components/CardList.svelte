<script lang="ts">
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	import { queryHref, type Schemas } from '#lib';
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import type { Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';
	import CardGrid from './CardGrid.svelte';
	import CardRow from './CardRow.svelte';
	import CardStack from './CardStack.svelte';
	import CardTile from './CardTile.svelte';
	import FilterBar from './FilterBar.svelte';

	let {
		cards,
		stats,
		filters = [],
		actions: extra,
		showToggle = true,
		view,
		columns
	}: {
		cards: Schemas['CardBase'][];
		stats?: ReadonlyMap<string, Schemas['CardStats']>;
		filters?: Filter[];
		actions?: Snippet;
		showToggle?: boolean;
		view?: 'grid' | 'rows';
		columns?: 2 | 3 | 4;
	} = $props();

	const mode = $derived(view ?? (page.url.searchParams.get('view') === 'rows' ? 'rows' : 'grid'));
	const views = [
		['grid', m.view_grid],
		['rows', m.view_rows]
	] as const;
</script>

<div class="px-4 py-3 sm:px-6">
	<FilterBar {filters}>
		{#snippet actions()}
			{@render extra?.()}
			{#if showToggle}
				<nav class="flex gap-1" aria-label={m.view()}>
					{#each views as [key, label] (key)}
						<a
							class={buttonVariants({ variant: mode === key ? 'secondary' : 'ghost', size: 'sm' })}
							aria-current={mode === key ? 'true' : undefined}
							href={queryHref(page.url, { view: key === 'grid' ? null : key })}
							data-sveltekit-reset="false">{label()}</a
						>
					{/each}
				</nav>
			{/if}
		{/snippet}
	</FilterBar>
</div>

{#if !cards.length}
	<p class="row text-sm text-muted-foreground">{m.no_cards()}</p>
{:else if mode === 'rows'}
	<CardStack>
		{#each cards as card (card.id)}
			<CardRow {card} stats={stats?.get(card.id)} />
		{/each}
	</CardStack>
{:else}
	<div>
		<CardGrid {columns}>
			{#each cards as card (card.id)}
				<li><CardTile {card} stats={stats?.get(card.id)} /></li>
			{/each}
		</CardGrid>
	</div>
{/if}
