<script lang="ts">
	import CardDetail from '#ui/components/CardDetail.svelte';
	import { m } from '$paraglide/messages.js';
	import type { Writes } from '#ui/writes.js';
	import type { cardData } from './data.js';

	let {
		data,
		writes
	}: { data: Awaited<ReturnType<typeof cardData>>; writes: Pick<Writes, 'vote'> } = $props();

	const card = $derived(data.card);
	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
</script>

<svelte:head>
	<title>{card.name ?? card.id} · {m.site_name()}</title>
	{#if card.description}<meta name="description" content={card.description} />{/if}
</svelte:head>

<CardDetail
	{card}
	stats={data.stats}
	versions={data.versions}
	decks={data.decks}
	matchups={data.matchups}
	{catalog}
	keywords={data.keywords}
	vote={writes.vote}
/>
