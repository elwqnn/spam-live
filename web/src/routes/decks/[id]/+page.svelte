<script lang="ts">
	import { writes } from '#lib';
	import DeckView from '@spam-live/ui/views/DeckView.svelte';
	import DeckImage from '#lib/components/export/DeckImage.svelte';
	import ExportImage from '#lib/components/export/ExportImage.svelte';
	import { credit, slug } from '#lib/export.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<DeckView {data} {writes}>
	{#snippet exportImage()}
		<ExportImage
			kind="deck"
			slug={slug(data.deck.name, String(data.deck.id))}
			credit={credit(data.cards.map((c) => c?.artist?.name))}
		>
			{#snippet children(size)}<DeckImage deck={data.deck} cards={data.cards} {size} />{/snippet}
		</ExportImage>
	{/snippet}
</DeckView>
