<script lang="ts">
	import { writes } from '#lib/host.js';
	import EditDeckView from '@spam-live/ui/views/EditDeckView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<EditDeckView {data} {writes} />
