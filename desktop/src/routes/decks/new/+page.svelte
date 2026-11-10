<script lang="ts">
	import { writes } from '#lib/host.js';
	import NewDeckView from '@spam-live/ui/views/NewDeckView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<NewDeckView {data} {writes} />
