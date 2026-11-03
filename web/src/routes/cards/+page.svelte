<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { fold, queryHref } from '#lib';
	import CardTile from '#lib/components/CardTile.svelte';
	import { ELEMENTS } from '#lib/components/ElementBadge.svelte';
	import { RARITIES } from '#lib/components/RarityBadge.svelte';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as NativeSelect from '#lib/components/ui/native-select/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const q = $derived(page.url.searchParams.get('q') ?? '');
	const element = $derived(page.url.searchParams.get('element') ?? '');
	const rarity = $derived(page.url.searchParams.get('rarity') ?? '');
	const sort = $derived(page.url.searchParams.get('sort') ?? 'order');

	const cards = $derived.by(() => {
		const needle = fold(q.trim());
		const list = data.cards.filter(
			(c) =>
				(!element || c.element === element) &&
				(!rarity || c.rarity === `rarity-${rarity}`) &&
				(!needle || fold(c.name ?? c.id).includes(needle))
		);
		if (sort === 'name')
			return list.sort((a, b) => (a.name ?? a.id).localeCompare(b.name ?? b.id, getLocale()));
		if (sort === 'hp') return list.sort((a, b) => b.hp - a.hp || a.order - b.order);
		return list.sort((a, b) => a.order - b.order);
	});

	function set(key: string, value: string) {
		goto(queryHref(page.url, { [key]: value === (key === 'sort' ? 'order' : '') ? null : value }), {
			replace: true,
			reset: false
		});
	}
</script>

<svelte:head>
	<title>{m.cards_title()} · {m.site_name()}</title>
</svelte:head>

<div class="grid gap-6">
	<h1 class="font-heading text-3xl font-medium tracking-tight sm:text-4xl">{m.cards_title()}</h1>

	<div class="flex flex-wrap gap-2">
		<Input
			type="search"
			class="w-full sm:w-64"
			placeholder={m.search()}
			aria-label={m.search()}
			value={q}
			oninput={(e) => set('q', e.currentTarget.value)}
		/>
		<NativeSelect.Root
			aria-label={m.filter_element()}
			value={element}
			onchange={(e) => set('element', e.currentTarget.value)}
		>
			<NativeSelect.Option value="">{m.all_elements()}</NativeSelect.Option>
			{#each Object.entries(ELEMENTS) as [key, [name]] (key)}
				<NativeSelect.Option value={key}>{name()}</NativeSelect.Option>
			{/each}
		</NativeSelect.Root>
		<NativeSelect.Root
			aria-label={m.filter_rarity()}
			value={rarity}
			onchange={(e) => set('rarity', e.currentTarget.value)}
		>
			<NativeSelect.Option value="">{m.all_rarities()}</NativeSelect.Option>
			{#each Object.entries(RARITIES) as [key, name] (key)}
				<NativeSelect.Option value={key.replace('rarity-', '')}>{name()}</NativeSelect.Option>
			{/each}
		</NativeSelect.Root>
		<NativeSelect.Root
			aria-label={m.sort()}
			value={sort}
			onchange={(e) => set('sort', e.currentTarget.value)}
		>
			<NativeSelect.Option value="order">{m.sort_order()}</NativeSelect.Option>
			<NativeSelect.Option value="name">{m.sort_name()}</NativeSelect.Option>
			<NativeSelect.Option value="hp">{m.sort_hp()}</NativeSelect.Option>
		</NativeSelect.Root>
	</div>

	{#if cards.length}
		<ul
			class="grid grid-cols-2 gap-3 rounded-2xl border board p-3 sm:grid-cols-[repeat(auto-fill,minmax(200px,1fr))] sm:gap-4 sm:p-5"
		>
			{#each cards as card, i (card.id)}
				<li><CardTile {card} index={i} /></li>
			{/each}
		</ul>
	{:else}
		<p class="text-sm text-muted-foreground">{m.no_cards()}</p>
	{/if}
</div>
