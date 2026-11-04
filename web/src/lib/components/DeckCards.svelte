<script lang="ts">
	import type { Snippet } from 'svelte';
	import { href, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import CardBadge from './CardBadge.svelte';
	import CardGrid from './CardGrid.svelte';
	import CardTile from './CardTile.svelte';

	let {
		cards,
		catalog,
		tiles = false,
		below
	}: {
		cards: string[];
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
		tiles?: boolean;
		below?: Snippet<[number]>;
	} = $props();
</script>

{#if tiles}
	<CardGrid columns={4}>
		{#each cards as id, i (id)}
			{@const card = catalog.get(id)}
			<li class="grid grid-cols-1 content-start">
				{#if card}
					<CardTile {card} />
				{:else}
					<div
						class="m-2 flex aspect-5/7 items-center justify-center border border-dashed p-3 text-center text-sm text-muted-foreground sm:m-3"
					>
						{m.unknown_card()}
						{id.slice(0, 8)}
					</div>
				{/if}
				{#if below}<div class="border-t p-3 sm:p-4">{@render below(i)}</div>{/if}
			</li>
		{/each}
	</CardGrid>
{:else}
	<ul class="flex flex-wrap gap-x-3 gap-y-1.5">
		{#each cards as id (id)}
			{@const card = catalog.get(id)}
			<li class="flex items-center gap-1.5 text-sm">
				{#if card}
					<CardBadge type="element" value={card.element} />
					<a class="hover:underline" href={href(`/cards/${id}`)}>{card.name ?? id}</a>
				{:else}
					<span class="text-muted-foreground">{m.unknown_card()} {id.slice(0, 8)}</span>
				{/if}
			</li>
		{/each}
	</ul>
{/if}
