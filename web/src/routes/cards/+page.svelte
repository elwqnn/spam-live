<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { fold, queryHref } from '#lib';
	import { ELEMENTS, RARITIES } from '#lib/components/CardBadge.svelte';
	import CardList from '#lib/components/CardList.svelte';
	import FilterMenu from '#lib/components/FilterMenu.svelte';
	import { Input } from '#lib/components/ui/input/index.js';
	import { selection, type Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const query = $derived(page.url.searchParams);
	const q = $derived(query.get('q') ?? '');
	const elements = $derived(selection(query.get('element')));
	const rarities = $derived(selection(query.get('rarity')).map((r) => `rarity-${r}`));
	const sort = $derived(query.get('sort') ?? 'order');
	const stats = $derived(new Map(data.stats.map((s) => [s.card, s])));

	const cards = $derived.by(() => {
		const needle = fold(q.trim());
		const list = data.cards.filter(
			(c) =>
				(!elements.length || elements.includes(c.element)) &&
				(!rarities.length || rarities.includes(c.rarity)) &&
				(!needle || fold(c.name ?? c.id).includes(needle))
		);
		if (sort === 'name')
			return list.sort((a, b) => (a.name ?? a.id).localeCompare(b.name ?? b.id, getLocale()));
		if (sort === 'hp') return list.sort((a, b) => b.hp - a.hp || a.order - b.order);
		return list.sort((a, b) => a.order - b.order);
	});

	const count = (key: 'element' | 'rarity', value: string) =>
		data.cards.filter((c) => c[key] === value).length;

	const filters: Filter[] = $derived([
		{
			id: 'element',
			label: m.filter_element(),
			multiple: true,
			options: Object.entries(ELEMENTS).map(([value, [label]]) => ({
				value,
				label: label(),
				count: count('element', value)
			}))
		},
		{
			id: 'rarity',
			label: m.filter_rarity(),
			multiple: true,
			options: Object.entries(RARITIES).map(([key, label]) => ({
				value: key.replace('rarity-', ''),
				label: label(),
				count: count('rarity', key)
			}))
		}
	]);

	const sorts: Filter = $derived({
		id: 'sort',
		label: m.sort(),
		default: 'order',
		options: [
			{ value: 'order', label: m.sort_order() },
			{ value: 'name', label: m.sort_name() },
			{ value: 'hp', label: m.sort_hp() }
		]
	});
</script>

<svelte:head>
	<title>{m.cards_title()} · {m.site_name()}</title>
</svelte:head>

<div class="flex flex-wrap items-end justify-between gap-4 row">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.cards_title()}
	</h1>
	<Input
		type="search"
		class="w-full sm:w-64"
		placeholder={m.search()}
		aria-label={m.search()}
		value={q}
		oninput={(e) =>
			goto(queryHref(page.url, { q: e.currentTarget.value || null }), {
				replace: true,
				reset: false
			})}
	/>
</div>

<CardList {cards} {stats} {filters}>
	{#snippet actions()}
		<FilterMenu filter={sorts} />
	{/snippet}
</CardList>
