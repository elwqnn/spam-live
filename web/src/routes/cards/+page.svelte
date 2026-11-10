<script lang="ts">
	import CardsView from '@spam-live/ui/views/CardsView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<CardsView {data} />
