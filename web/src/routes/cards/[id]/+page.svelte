<script lang="ts">
	import { writes } from '#lib';
	import CardView from '@spam-live/ui/views/CardView.svelte';
	import CardImage from '#lib/components/export/CardImage.svelte';
	import ExportImage from '#lib/components/export/ExportImage.svelte';
	import { credit, slug } from '#lib/export.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<CardView {data} {writes}>
	{#snippet exportImage()}
		<ExportImage
			kind="card"
			slug={slug(data.card.name ?? '', data.card.id)}
			credit={credit([data.card.artist?.name])}
		>
			{#snippet children(size)}<CardImage card={data.card} stats={data.stats} {size} />{/snippet}
		</ExportImage>
	{/snippet}
</CardView>
