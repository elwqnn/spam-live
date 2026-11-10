<script lang="ts">
	import { writes } from '#lib';
	import DecksView from '@spam-live/ui/views/DecksView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<DecksView {data} {writes} />
