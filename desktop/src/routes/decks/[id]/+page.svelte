<script lang="ts">
	import { writes } from '#lib/host.js';
	import DeckView from '@spam-live/ui/views/DeckView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<DeckView {data} {writes} />
