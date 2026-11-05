<script lang="ts">
	import { page } from '$app/state';
	import { queryHref } from '#lib';
	import CardRow from '#lib/components/CardRow.svelte';
	import CardStack from '#lib/components/CardStack.svelte';
	import DeckKeyTile from '#lib/components/DeckKeyTile.svelte';
	import FilterBar from '#lib/components/FilterBar.svelte';
	import FilterMenu from '#lib/components/FilterMenu.svelte';
	import MetaCoverage from '#lib/components/MetaCoverage.svelte';
	import WinRate from '#lib/components/WinRate.svelte';
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import type { Filter } from '#lib/filter.js';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import TrendMenu from '#lib/components/TrendMenu.svelte';
	import { versionFilter } from '#lib/components/VersionSelect.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const tab = $derived(page.url.searchParams.get('tab') === 'cards' ? 'cards' : 'decks');
	const link = (active: boolean) =>
		buttonVariants({ variant: active ? 'secondary' : 'ghost', size: 'sm' });
	const filters: Filter[] = $derived([versionFilter(data.versions)]);
	const sorts: Filter = $derived({
		id: 'sort',
		label: m.sort(),
		default: 'winrate',
		options: [
			{ value: 'winrate', label: m.win_rate() },
			{ value: 'games', label: m.games() }
		]
	});
</script>

<svelte:head>
	<title>{m.stats_title()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="grid gap-1 row">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.stats_title()}
	</h1>
	<p class="text-sm text-muted-foreground">{m.stats_intro()}</p>
</TitleRow>

<div class="px-(--gutter) py-3">
	<FilterBar {filters}>
		{#snippet actions()}
			{#if tab === 'decks'}<FilterMenu filter={sorts} />{:else}<TrendMenu />{/if}
		{/snippet}
	</FilterBar>
</div>

<MetaCoverage cards={data.cards} stats={data.cardStats} />

<nav class="flex gap-1 px-(--gutter) py-3">
	<a
		class={link(tab === 'decks')}
		aria-current={tab === 'decks' ? 'page' : undefined}
		href={queryHref(page.url, { tab: null })}>{m.tab_decks()}</a
	>
	<a
		class={link(tab === 'cards')}
		aria-current={tab === 'cards' ? 'page' : undefined}
		href={queryHref(page.url, { tab: 'cards' })}>{m.tab_cards()}</a
	>
</nav>

{#if tab === 'decks'}
	{#if data.decks.length}
		<div>
			<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
				{#each data.decks as deck, i (deck.cards.join())}
					<DeckKeyTile {deck} rank={i + 1} {catalog} />
				{/each}
			</ul>
		</div>
	{:else}
		<p class="row text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
{:else if data.cardStats.length}
	<CardStack>
		{#each data.cardStats as row (row.card)}
			{@const card = catalog.get(row.card)}
			{#if card}
				<CardRow {card} stats={row} />
			{:else}
				<li class="flex items-center justify-between gap-3 px-(--gutter) py-3">
					<span class="text-sm text-muted-foreground"
						>{m.unknown_card()} {row.card.slice(0, 8)}</span
					>
					<WinRate value={row.winRate} />
				</li>
			{/if}
		{/each}
	</CardStack>
{:else}
	<p class="row text-sm text-muted-foreground">{m.no_stats()}</p>
{/if}
