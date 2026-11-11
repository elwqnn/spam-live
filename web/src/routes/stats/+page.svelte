<script lang="ts">
	import { page } from '$app/state';
	import StatsView from '@spam-live/ui/views/StatsView.svelte';
	import ExportImage from '#lib/components/export/ExportImage.svelte';
	import TierImage from '#lib/components/export/TierImage.svelte';
	import { slug } from '#lib/export.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const version = $derived(page.url.searchParams.get('version') ?? data.versions[0] ?? 'all');
</script>

<StatsView {data}>
	{#snippet exportImage()}
		<ExportImage kind="tiers" slug={slug(version)}>
			<TierImage stats={data.stats} {catalog} />
		</ExportImage>
	{/snippet}
</StatsView>
