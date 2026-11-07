<script lang="ts">
	import CardDetail from '#lib/components/CardDetail.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

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
/>
