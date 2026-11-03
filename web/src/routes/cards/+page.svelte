<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { fold, href, queryHref } from '#lib';
	import ElementBadge, { ELEMENTS } from '#lib/components/ElementBadge.svelte';
	import RarityBadge, { RARITIES } from '#lib/components/RarityBadge.svelte';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as NativeSelect from '#lib/components/ui/native-select/index.js';
	import * as Table from '#lib/components/ui/table/index.js';
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

<div class="grid gap-4">
	<h1 class="font-heading text-2xl font-semibold">{m.cards_title()}</h1>

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
			{#each Object.entries(RARITIES) as [key, [name]] (key)}
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
		<Table.Root>
			<Table.Header>
				<Table.Row>
					<Table.Head>{m.sort_name()}</Table.Head>
					<Table.Head>{m.filter_element()}</Table.Head>
					<Table.Head class="hidden sm:table-cell">{m.filter_rarity()}</Table.Head>
					<Table.Head class="text-right">{m.hp()}</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each cards as card (card.id)}
					<Table.Row>
						<Table.Cell class="whitespace-normal">
							<a class="font-medium hover:underline" href={href(`/cards/${card.id}`)}
								>{card.name ?? card.id}</a
							>
						</Table.Cell>
						<Table.Cell><ElementBadge element={card.element} /></Table.Cell>
						<Table.Cell class="hidden sm:table-cell"
							><RarityBadge rarity={card.rarity} /></Table.Cell
						>
						<Table.Cell class="text-right tabular-nums">{card.hp}</Table.Cell>
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>
	{:else}
		<p class="text-sm text-muted-foreground">{m.no_cards()}</p>
	{/if}
</div>
