<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { page } from '$app/state';
	import { queryHref, type Schemas } from '#lib';
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import type { Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';
	import { cn } from '#lib/utils.js';
	import CardGrid from './CardGrid.svelte';
	import CardRow from './CardRow.svelte';
	import CardStack from './CardStack.svelte';
	import CardTile from './CardTile.svelte';
	import FilterBar from './FilterBar.svelte';
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
	const views = [
		['grid', m.view_grid],
		['rows', m.view_rows]
	] as const;
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

<div class="px-(--gutter) py-3">
	<FilterBar {filters}>
		{#snippet actions()}
			{@render extra?.()}
			{#if stats && mode === 'grid'}
				<label class="flex items-center gap-2 text-sm">
					{m.stats()}
					<Switch
						checked={shown}
						class="in-[.stats-off]:border-foreground/25! in-[.stats-off]:bg-transparent! in-[.stats-off]:*:translate-x-0! in-[.stats-off]:*:bg-foreground!"
						onclick={toggleStats}
					/>
				</label>
			{/if}
			{#if showToggle}
				<div
					role="group"
					aria-label={m.view()}
					class="flex h-8 overflow-hidden rounded-sm border border-foreground/25"
				>
					{#each views as [key, label] (key)}
						<a
							class={cn(
								buttonVariants({ variant: mode === key ? 'secondary' : 'ghost', size: 'sm' }),
								'h-auto rounded-none border-0 focus-visible:ring-inset',
								mode !== key && 'text-muted-foreground'
							)}
							aria-current={mode === key ? 'true' : undefined}
							href={queryHref(page.url, { view: key === 'grid' ? null : key })}
							data-sveltekit-reset="false">{label()}</a
						>
					{/each}
				</div>
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
				<li>
					<CardTile {card} stats={shown && stats ? (stats.get(card.id) ?? empty) : undefined} />
				</li>
			{/each}
		</CardGrid>
	</div>
{/if}
