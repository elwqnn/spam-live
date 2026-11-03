<script lang="ts">
	import { page } from '$app/state';
	import { href, percent, queryHref } from '#lib';
	import DeckCards from '#lib/components/DeckCards.svelte';
	import ElementBadge from '#lib/components/ElementBadge.svelte';
	import VersionSelect from '#lib/components/VersionSelect.svelte';
	import WinRate from '#lib/components/WinRate.svelte';
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import * as Table from '#lib/components/ui/table/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const tab = $derived(page.url.searchParams.get('tab') === 'cards' ? 'cards' : 'decks');
	const link = (active: boolean) =>
		buttonVariants({ variant: active ? 'secondary' : 'ghost', size: 'sm' });
</script>

<svelte:head>
	<title>{m.stats_title()} · {m.site_name()}</title>
</svelte:head>

<div class="grid gap-4">
	<div class="grid gap-1">
		<h1 class="font-heading text-3xl font-medium tracking-tight sm:text-4xl">{m.stats_title()}</h1>
		<p class="text-sm text-muted-foreground">{m.stats_intro()}</p>
	</div>

	<div class="flex flex-wrap items-center justify-between gap-2">
		<nav class="flex gap-1">
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
		<VersionSelect versions={data.versions} />
	</div>

	{#if tab === 'decks'}
		<div class="flex flex-wrap items-center gap-1">
			<span class="mr-1 mono-label text-muted-foreground">{m.sort()}</span>
			<a
				class={link(data.sort === 'winrate')}
				href={queryHref(page.url, { sort: null })}
				data-sveltekit-reset="false">{m.win_rate()}</a
			>
			<a
				class={link(data.sort === 'games')}
				href={queryHref(page.url, { sort: 'games' })}
				data-sveltekit-reset="false">{m.games()}</a
			>
		</div>
		{#if data.decks.length}
			<div class="overflow-hidden rounded-xl border bg-card">
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head class="pl-4 sm:pl-5">{m.deck()}</Table.Head>
							<Table.Head class="text-right whitespace-normal">{m.games()}</Table.Head>
							<Table.Head class="pr-4 text-right whitespace-normal sm:pr-5"
								>{m.win_rate()}</Table.Head
							>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each data.decks as deck (deck.cards.join())}
							<Table.Row>
								<Table.Cell class="min-w-40 py-3.5 pl-4 whitespace-normal sm:min-w-48 sm:pl-5">
									<DeckCards cards={deck.cards} {catalog} />
								</Table.Cell>
								<Table.Cell class="text-right font-mono text-[13px] tabular-nums"
									>{deck.games}</Table.Cell
								>
								<Table.Cell class="pr-4 sm:pr-5"><WinRate value={deck.winRate} /></Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			</div>
		{:else}
			<p class="text-sm text-muted-foreground">{m.no_stats()}</p>
		{/if}
	{:else if data.cardStats.length}
		<div class="overflow-hidden rounded-xl border bg-card">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head class="pl-4 sm:pl-5">{m.card()}</Table.Head>
						<Table.Head class="text-right whitespace-normal">{m.games()}</Table.Head>
						<Table.Head class="hidden text-right sm:table-cell">{m.play_rate()}</Table.Head>
						<Table.Head class="pr-4 text-right whitespace-normal sm:pr-5">{m.win_rate()}</Table.Head
						>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each data.cardStats as row (row.card)}
						{@const card = catalog.get(row.card)}
						<Table.Row>
							<Table.Cell class="pl-4 whitespace-normal sm:pl-5">
								{#if card}
									<span class="flex items-center gap-1.5">
										<ElementBadge element={card.element} />
										<a class="hover:underline" href={href(`/cards/${row.card}`)}
											>{card.name ?? card.id}</a
										>
									</span>
								{:else}
									<span class="text-muted-foreground"
										>{m.unknown_card()} {row.card.slice(0, 8)}</span
									>
								{/if}
							</Table.Cell>
							<Table.Cell class="text-right font-mono text-[13px] tabular-nums"
								>{row.games}</Table.Cell
							>
							<Table.Cell class="hidden text-right font-mono text-[13px] tabular-nums sm:table-cell"
								>{percent(row.playRate)}</Table.Cell
							>
							<Table.Cell class="pr-4 sm:pr-5"
								><WinRate value={row.winRate} element={card?.element} /></Table.Cell
							>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</div>
	{:else}
		<p class="text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
</div>
