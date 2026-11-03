<script lang="ts">
	import { href, type CardMap } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import ElementBadge from './ElementBadge.svelte';

	let { cards, catalog }: { cards: string[]; catalog: CardMap } = $props();
</script>

<ul class="flex flex-wrap gap-x-3 gap-y-1">
	{#each cards as id (id)}
		{@const card = catalog.get(id)}
		<li class="flex items-center gap-1.5 text-sm">
			{#if card}
				<ElementBadge element={card.element} />
				<a class="hover:underline" href={href(`/cards/${id}`)}>{card.name ?? id}</a>
			{:else}
				<span class="text-muted-foreground">{m.unknown_card()} {id.slice(0, 8)}</span>
			{/if}
		</li>
	{/each}
</ul>
