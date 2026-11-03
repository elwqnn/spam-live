<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { href, percent, queryHref } from '#lib';
	import Author from '#lib/components/Author.svelte';
	import DeckCards from '#lib/components/DeckCards.svelte';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import * as NativeSelect from '#lib/components/ui/native-select/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const options = $derived(
		data.cards.toSorted((a, b) => (a.name ?? a.id).localeCompare(b.name ?? b.id, getLocale()))
	);
	const reset = { offset: null, before: null };
	const link = (active: boolean) =>
		buttonVariants({ variant: active ? 'secondary' : 'ghost', size: 'sm' });
	const sorts = [
		['votes', m.sort_votes],
		['recent', m.sort_recent],
		['winrate', m.win_rate]
	] as const;
</script>

<svelte:head>
	<title>{m.decks_title()} · {m.site_name()}</title>
</svelte:head>

<div class="grid gap-4">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<h1 class="font-heading text-2xl font-semibold">{m.decks_title()}</h1>
		<Button href={href('/decks/new')}>{m.new_deck()}</Button>
	</div>

	{#if data.me}
		<nav class="flex gap-1">
			<a
				class={link(!data.mine)}
				aria-current={data.mine ? undefined : 'page'}
				href={queryHref(page.url, { ...reset, tab: null })}>{m.all_decks()}</a
			>
			<a
				class={link(data.mine)}
				aria-current={data.mine ? 'page' : undefined}
				href={queryHref(page.url, { ...reset, tab: 'mine' })}>{m.my_decks()}</a
			>
		</nav>
	{/if}

	<div class="flex flex-wrap items-center justify-between gap-2">
		<div class="flex flex-wrap items-center gap-1 text-sm">
			<span class="text-muted-foreground">{m.sort()}:</span>
			{#each sorts as [key, label] (key)}
				<a
					class={link(data.sort === key)}
					aria-current={data.sort === key ? 'true' : undefined}
					href={queryHref(page.url, { ...reset, sort: key === 'votes' ? null : key })}
					data-sveltekit-reset="false">{label()}</a
				>
			{/each}
		</div>
		<label class="flex min-w-0 items-center gap-2 text-sm">
			<span class="text-muted-foreground">{m.card()}</span>
			<NativeSelect.Root
				class="max-w-56"
				value={data.card ?? ''}
				onchange={(e) =>
					goto(queryHref(page.url, { ...reset, card: e.currentTarget.value || null }), {
						reset: false
					})}
			>
				<NativeSelect.Option value="">{m.all_cards()}</NativeSelect.Option>
				{#each options as card (card.id)}
					<NativeSelect.Option value={card.id}>{card.name ?? card.id}</NativeSelect.Option>
				{/each}
			</NativeSelect.Root>
		</label>
	</div>

	{#if data.decks.length}
		<ul class="grid gap-3">
			{#each data.decks as deck (deck.id)}
				<li class="grid gap-2 rounded-xl border p-3">
					<div class="flex flex-wrap items-center gap-x-3 gap-y-1">
						<a class="font-medium wrap-anywhere hover:underline" href={href(`/decks/${deck.id}`)}
							>{deck.name}</a
						>
						{#if deck.status === 'draft'}<Badge variant="secondary">{m.draft()}</Badge>{/if}
						<span class="flex min-w-0 text-sm text-muted-foreground">
							<Author name={deck.authorName} avatar={deck.authorAvatar} />
						</span>
					</div>
					<DeckCards cards={deck.cards} {catalog} />
					<p class="text-xs text-muted-foreground tabular-nums">
						{m.votes()}: {deck.votes} · {m.win_rate()}: {percent(deck.winRate)} · {m.games()}: {deck.games}
					</p>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="text-sm text-muted-foreground">{m.no_decks()}</p>
	{/if}

	{#if data.prev || data.next}
		<nav class="flex justify-between gap-2" aria-label={m.pages()}>
			{#if data.prev}
				<a class={link(false)} href={queryHref(page.url, data.prev)}
					>{data.sort === 'recent' ? m.first_page() : m.previous_page()}</a
				>
			{:else}
				<span></span>
			{/if}
			{#if data.next}
				<a class={link(false)} href={queryHref(page.url, data.next)}>{m.next_page()}</a>
			{/if}
		</nav>
	{/if}
</div>
