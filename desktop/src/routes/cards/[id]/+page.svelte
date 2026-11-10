<script lang="ts">
	import { writes } from '#lib/host.js';
	import CardView from '@spam-live/ui/views/CardView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<CardView {data} {writes} />
