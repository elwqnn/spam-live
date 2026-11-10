<script lang="ts">
	import StatsView from '@spam-live/ui/views/StatsView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<StatsView {data} />
