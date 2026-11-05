<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { href, queryHref } from '#lib';
	import DeckTile from '#lib/components/DeckTile.svelte';
	import FilterBar from '#lib/components/FilterBar.svelte';
	import FilterMenu from '#lib/components/FilterMenu.svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import type { Filter } from '#lib/filter.js';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import { promote } from '#lib/promote.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const promoted = $derived(data.prev ? [] : promote(data.decks));
	const reset = { offset: null, before: null };
	const kept = $derived(
		[...page.url.searchParams].filter(([key]) => !['q', 'offset', 'before'].includes(key))
	);
	function search(e: SubmitEvent & { currentTarget: HTMLFormElement }) {
		e.preventDefault();
		const q = String(new FormData(e.currentTarget).get('q') ?? '').trim();
		goto(queryHref(page.url, { ...reset, q: [...q].length >= 2 ? q : null }), { reset: false });
	}
	const link = (active: boolean) =>
		buttonVariants({ variant: active ? 'secondary' : 'ghost', size: 'sm' });
	const filters: Filter[] = $derived([
		{
			id: 'card',
			label: m.card(),
			default: 'all',
			options: [
				{ value: 'all', label: m.all_cards() },
				...data.cards
					.map((c) => ({ value: c.id, label: c.name ?? c.id }))
					.sort((a, b) => a.label.localeCompare(b.label, getLocale()))
			]
		}
	]);
	const sorts: Filter = $derived({
		id: 'sort',
		label: m.sort(),
		default: 'votes',
		options: [
			{ value: 'votes', label: m.sort_votes() },
			{ value: 'recent', label: m.sort_recent() },
			{ value: 'winrate', label: m.win_rate() }
		]
	});
</script>

<svelte:head>
	<title>{m.decks_title()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="flex flex-wrap items-center justify-between gap-2 row">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.decks_title()}
	</h1>
	<Button href={href('/decks/new')}>{m.new_deck()}</Button>
</TitleRow>

<div class="flex flex-wrap items-center gap-2 px-(--gutter) py-3">
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
	<form method="GET" role="search" class="w-full sm:w-56" onsubmit={search}>
		{#each kept as [key, value], i (i)}<input type="hidden" name={key} {value} />{/each}
		<Input
			type="search"
			name="q"
			maxlength={64}
			class="h-8"
			placeholder={m.search()}
			aria-label={m.search()}
			value={page.url.searchParams.get('q') ?? ''}
		/>
	</form>
	<div class="min-w-0 flex-1">
		<FilterBar {filters} {reset}>
			{#snippet actions()}
				<FilterMenu filter={sorts} {reset} />
			{/snippet}
		</FilterBar>
	</div>
</div>

{#if data.decks.length}
	<div>
		<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
			{#each data.decks as deck, i (deck.id)}
				<DeckTile {deck} {catalog} reasons={promoted[i]?.reasons} large={promoted[i]?.large} />
			{/each}
		</ul>
	</div>
{:else}
	<p class="row text-sm text-muted-foreground">{m.no_decks()}</p>
{/if}

{#if data.prev || data.next}
	<nav class="flex justify-between gap-2 px-(--gutter) py-3" aria-label={m.pages()}>
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
