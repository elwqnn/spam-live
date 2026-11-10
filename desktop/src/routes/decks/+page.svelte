<script lang="ts">
	import { writes } from '#lib/host.js';
	import DecksView from '@spam-live/ui/views/DecksView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<DecksView {data} {writes} />
