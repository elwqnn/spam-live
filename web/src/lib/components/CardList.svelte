<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { page } from '$app/state';
	import type { Schemas } from '#lib';
	import type { Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';
	import CardGrid from './CardGrid.svelte';
	import CardRow from './CardRow.svelte';
	import CardStack from './CardStack.svelte';
	import CardTile from './CardTile.svelte';
	import FilterChips from './FilterChips.svelte';
	import Switch from './Switch.svelte';

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
	const views: Filter = $derived({
		id: 'view',
		label: m.view(),
		default: 'grid',
		options: [
			{ value: 'grid', label: m.view_grid() },
			{ value: 'rows', label: m.view_rows() }
		]
	});
	const empty = { games: 0, playRate: 0, winRate: null };

	let shown = $state(true);

	onMount(() => {
		shown = !document.documentElement.classList.contains('stats-off');
	});

	function toggleStats() {
		shown = !shown;
		document.documentElement.classList.toggle('stats-off', !shown);
		try {
			localStorage.setItem('cardStats', shown ? 'on' : 'off');
		} catch {
			return;
		}
	}
</script>

<div class="px-(--gutter) py-4">
	<div role="group" aria-label={m.filters()} class="grid gap-3">
		{#each filters as filter (filter.id)}
			<FilterChips {filter} class="sm:grid sm:grid-cols-[5rem_minmax(0,1fr)] sm:items-start" />
		{/each}
		<div class="flex flex-wrap items-center gap-x-6 gap-y-3">
			{@render extra?.()}
			<div class="ml-auto flex flex-wrap items-center gap-x-6 gap-y-3">
				{#if stats && mode === 'grid'}
					<label class="flex items-center gap-3">
						<span class="eyebrow text-muted-foreground">{m.stats()}</span>
						<Switch
							checked={shown}
							class="in-[.stats-off]:border-foreground/25! in-[.stats-off]:bg-transparent! in-[.stats-off]:*:translate-x-0! in-[.stats-off]:*:bg-foreground!"
							onclick={toggleStats}
						/>
					</label>
				{/if}
				{#if showToggle}
					<FilterChips filter={views} selected={mode} />
				{/if}
			</div>
		</div>
	</div>
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
				<li>
					<CardTile {card} stats={shown && stats ? (stats.get(card.id) ?? empty) : undefined} />
				</li>
			{/each}
		</CardGrid>
	</div>
{/if}
