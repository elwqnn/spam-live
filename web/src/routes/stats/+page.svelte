<script lang="ts">
	import { page } from '$app/state';
	import { queryHref } from '#lib';
	import CardRow from '#lib/components/CardRow.svelte';
	import CardStack from '#lib/components/CardStack.svelte';
	import DeckCards from '#lib/components/DeckCards.svelte';
	import FilterBar from '#lib/components/FilterBar.svelte';
	import FilterMenu from '#lib/components/FilterMenu.svelte';
	import MetaCoverage from '#lib/components/MetaCoverage.svelte';
	import WinRate from '#lib/components/WinRate.svelte';
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import * as Table from '#lib/components/ui/table/index.js';
	import type { Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const tab = $derived(page.url.searchParams.get('tab') === 'cards' ? 'cards' : 'decks');
	const link = (active: boolean) =>
		buttonVariants({ variant: active ? 'secondary' : 'ghost', size: 'sm' });
	const filters: Filter[] = $derived([
		{
			id: 'version',
			label: m.version(),
			default: data.versions[0],
			options: [
				...data.versions.map((v) => ({ value: v, label: v })),
				{ value: 'all', label: m.all_versions() }
			]
		}
	]);
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

<div class="grid gap-1 row">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.stats_title()}
	</h1>
	<p class="text-sm text-muted-foreground">{m.stats_intro()}</p>
</div>

<div class="px-4 py-3 sm:px-6">
	<FilterBar {filters}>
		{#snippet actions()}
			{#if tab === 'decks'}<FilterMenu filter={sorts} />{/if}
		{/snippet}
	</FilterBar>
</div>

<MetaCoverage cards={data.cards} stats={data.cardStats} />

<nav class="flex gap-1 px-4 py-3 sm:px-6">
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
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head class="pl-4 sm:pl-6">{m.deck()}</Table.Head>
						<Table.Head class="text-right whitespace-normal">{m.games()}</Table.Head>
						<Table.Head class="pr-4 text-right whitespace-normal sm:pr-6">{m.win_rate()}</Table.Head
						>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each data.decks as deck (deck.cards.join())}
						<Table.Row>
							<Table.Cell class="min-w-40 py-3.5 pl-4 whitespace-normal sm:min-w-48 sm:pl-6">
								<DeckCards cards={deck.cards} {catalog} />
							</Table.Cell>
							<Table.Cell class="text-right text-[13px] tabular-nums">{deck.games}</Table.Cell>
							<Table.Cell class="pr-4 sm:pr-6"><WinRate value={deck.winRate} /></Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
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
				<li class="flex items-center justify-between gap-3 px-4 py-3 sm:px-6">
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
